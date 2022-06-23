use proc_macro2::{Ident, Literal, Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::{
    parse::{Parse, ParseStream},
    parse_quote,
    punctuated::Punctuated,
    spanned::Spanned,
    token::Comma,
    Data, DataStruct, DeriveInput, Error, Fields, FieldsNamed, GenericParam, LitInt, Path, Type,
};

pub use syn;

#[macro_export]
macro_rules! implement {
    ($path:expr) => {
        #[proc_macro_derive(ShaderType, attributes(align, size))]
        pub fn derive_shader_type(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
            let input = $crate::syn::parse_macro_input!(input as $crate::syn::DeriveInput);
            let expanded = encase_derive_impl::derive_shader_type(input, &$path);
            proc_macro::TokenStream::from(expanded)
        }
    };
}

fn get_named_struct_fields(data: &syn::Data) -> syn::Result<&FieldsNamed> {
    match data {
        Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) if !fields.named.is_empty() => Ok(fields),
        _ => Err(Error::new(
            Span::call_site(),
            "Only non empty structs with named fields are supported!",
        )),
    }
}

struct FieldData {
    pub field: syn::Field,
    pub size: Option<(u32, Span)>,
    pub align: Option<(u32, Span)>,
}

impl FieldData {
    fn alignment(&self, root: &Path) -> TokenStream {
        if let Some((alignment, _)) = self.align {
            let alignment = Literal::u64_suffixed(alignment as u64);
            quote! {
                #root::AlignmentValue::new(#alignment)
            }
        } else {
            let ty = &self.field.ty;
            quote! {
                <#ty as #root::ShaderType>::METADATA.alignment()
            }
        }
    }

    fn size(&self, root: &Path) -> TokenStream {
        if let Some((size, _)) = self.size {
            let size = Literal::u64_suffixed(size as u64);
            quote! {
                #size
            }
        } else {
            let ty = &self.field.ty;
            quote! {
                <#ty as #root::ShaderSize>::SHADER_SIZE.get()
            }
        }
    }

    fn min_size(&self, root: &Path) -> TokenStream {
        if let Some((size, _)) = self.size {
            let size = Literal::u64_suffixed(size as u64);
            quote! {
                #size
            }
        } else {
            let ty = &self.field.ty;
            quote! {
                <#ty as #root::ShaderType>::METADATA.min_size().get()
            }
        }
    }

    fn extra_padding(&self, root: &Path) -> Option<TokenStream> {
        self.size.as_ref().map(|(size, _)| {
            let size = Literal::u64_suffixed(*size as u64);
            let ty = &self.field.ty;
            let original_size = quote! { <#ty as #root::ShaderSize>::SHADER_SIZE.get() };
            quote!(#size.saturating_sub(#original_size))
        })
    }

    fn ident(&self) -> &Ident {
        self.field.ident.as_ref().unwrap()
    }
}

struct AlignmentAttr(u32);

impl Parse for AlignmentAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        match input
            .parse::<LitInt>()
            .and_then(|lit| lit.base10_parse::<u32>())
        {
            Ok(num) if num.is_power_of_two() => Ok(Self(num)),
            _ => Err(syn::Error::new(
                input.span(),
                "expected a power of 2 u32 literal",
            )),
        }
    }
}

struct StaticSizeAttr(u32);

impl Parse for StaticSizeAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        match input
            .parse::<LitInt>()
            .and_then(|lit| lit.base10_parse::<u32>())
        {
            Ok(num) => Ok(Self(num)),
            _ => Err(syn::Error::new(input.span(), "expected u32 literal")),
        }
    }
}

enum SizeAttr {
    Static(StaticSizeAttr),
    Runtime,
}

impl Parse for SizeAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        match input.parse::<StaticSizeAttr>() {
            Ok(static_size) => Ok(SizeAttr::Static(static_size)),
            _ => match input.parse::<Path>() {
                Ok(ident) if ident.is_ident("runtime") => Ok(SizeAttr::Runtime),
                _ => Err(syn::Error::new(
                    input.span(),
                    "expected u32 literal or `runtime` identifier",
                )),
            },
        }
    }
}

struct Errors {
    inner: Option<Error>,
}

impl Errors {
    fn new() -> Self {
        Self { inner: None }
    }

    fn append(&mut self, err: Error) {
        if let Some(ex_error) = &mut self.inner {
            ex_error.combine(err);
        } else {
            self.inner.replace(err);
        }
    }

    fn into_compile_error(self) -> Option<TokenStream> {
        self.inner.map(|e| e.into_compile_error())
    }
}

pub fn derive_shader_type(input: DeriveInput, root: &Path) -> TokenStream {
    let root = &parse_quote!(#root::private);

    let fields = match get_named_struct_fields(&input.data) {
        Ok(fields) => fields,
        Err(e) => return e.into_compile_error(),
    };

    let last_field_index = fields.named.len() - 1;

    let mut errors = Errors::new();

    let mut is_runtime_sized = false;

    let field_data: Vec<_> = fields
        .named
        .iter()
        .enumerate()
        .map(|(i, field)| {
            let mut data = FieldData {
                field: field.clone(),
                size: None,
                align: None,
            };
            for attr in &field.attrs {
                let span = attr.tokens.span();
                if attr.path.is_ident("align") {
                    let res = attr.parse_args::<AlignmentAttr>();
                    let res = res.map_err(|err| syn::Error::new(span, err));
                    match res {
                        Ok(val) => data.align = Some((val.0, span)),
                        Err(err) => errors.append(err),
                    }
                } else if attr.path.is_ident("size") {
                    let res = if i == last_field_index {
                        attr.parse_args::<SizeAttr>().map(|val| match val {
                            SizeAttr::Runtime => {
                                is_runtime_sized = true;
                                None
                            }
                            SizeAttr::Static(size) => Some((size.0, span)),
                        })
                    } else {
                        attr.parse_args::<StaticSizeAttr>()
                            .map(|val| Some((val.0, span)))
                    };
                    let res = res.map_err(|err| syn::Error::new(span, err));
                    match res {
                        Ok(val) => data.size = val,
                        Err(err) => errors.append(err),
                    }
                }
            }
            data
        })
        .collect();

    let mut found = false;
    let size_hint: &Path = &parse_quote!(#root::ArrayLength);
    for field in &fields.named {
        // TODO: rethink how to check type equality here
        match &field.ty {
            Type::Path(path)
                if path.path.segments.last().unwrap().ident
                    == size_hint.segments.last().unwrap().ident =>
            {
                if found {
                    let err = syn::Error::new(
                        field.ty.span(),
                        "only one field can use the `ArrayLength` type!",
                    );
                    errors.append(err)
                } else {
                    if !is_runtime_sized {
                        let err = syn::Error::new(
                                field.ty.span(),
                                "`ArrayLength` type can only be used within a struct containing a runtime-sized array marked as `#[size(runtime)]`!",
                            );
                        errors.append(err)
                    }
                    found = true;
                }
            }
            _ => {}
        }
    }

    if let Some(ts) = errors.into_compile_error() {
        return ts;
    }

    let nr_of_fields = &Literal::usize_suffixed(field_data.len());

    let field_trait_constraints = generate_field_trait_constraints(
        &input,
        &field_data,
        if is_runtime_sized {
            quote!(#root::ShaderType + #root::RuntimeSizedArray)
        } else {
            quote!(#root::ShaderType + #root::ShaderSize)
        },
        quote!(#root::ShaderType + #root::ShaderSize),
    );

    let mut lifetimes = input.generics.clone();
    lifetimes.params = lifetimes
        .params
        .into_iter()
        .filter(|param| matches!(param, GenericParam::Lifetime(_)))
        .collect::<Punctuated<GenericParam, Comma>>();

    let align_check = {
        let (impl_generics, _, _) = lifetimes.split_for_impl();
        field_data
            .iter()
            .filter_map(|data| data.align.as_ref().map(|align| (&data.field.ty, align)))
            .map(move |(ty, (align, span))| {
                let align = Literal::u64_suffixed(*align as u64);
                quote_spanned! {*span=>
                    const _: () = {
                        #[track_caller]
                        #[allow(clippy::extra_unused_lifetimes)]
                        const fn check #impl_generics () {
                            let alignment = <#ty as #root::ShaderType>::METADATA.alignment().get();
                            #root::concat_assert!(
                                alignment <= #align,
                                "align attribute value must be at least ", alignment, " (field's type alignment)"
                            )
                        }
                        check();
                    };
                }
            })
    };

    let size_check = {
        let (impl_generics, _, _) = lifetimes.split_for_impl();
        field_data
            .iter()
            .filter_map(|data| data.size.as_ref().map(|size| (&data.field.ty, size)))
            .map(move |(ty, (size, span))| {
                let size = Literal::u64_suffixed(*size as u64);
                quote_spanned! {*span=>
                    const _: () = {
                        #[track_caller]
                        #[allow(clippy::extra_unused_lifetimes)]
                        const fn check #impl_generics () {
                            let size = <#ty as #root::ShaderSize>::SHADER_SIZE.get();
                            #root::concat_assert!(
                                size <= #size,
                                "size attribute value must be at least ", size, " (field's type size)"
                            )
                        }
                        check();
                    };
                }
            })
    };

    let uniform_check = field_data.iter().enumerate().map(|(i, data)| {
        let ty = &data.field.ty;
        let ty_check = quote_spanned! {ty.span()=>
            <#ty as #root::ShaderType>::UNIFORM_COMPAT_ASSERT()
        };
        let ident = data.ident();
        let name = ident.to_string();
        let field_offset_check = quote_spanned! {ident.span()=>
            if let ::core::option::Option::Some(min_alignment) =
                <#ty as #root::ShaderType>::METADATA.uniform_min_alignment()
            {
                let offset = <Self as #root::ShaderType>::METADATA.offset(#i);

                #root::concat_assert!(
                    min_alignment.is_aligned(offset),
                    "offset of field '", #name, "' must be a multiple of ", min_alignment.get(),
                    " (current offset: ", offset, ")"
                )
            }
        };
        let field_offset_diff = if i != 0 {
            let prev_field = &field_data[i - 1];
            let prev_field_ty = &prev_field.field.ty;
            let prev_ident_name = prev_field.ident().to_string();
            quote_spanned! {ident.span()=>
                if let ::core::option::Option::Some(min_alignment) =
                    <#prev_field_ty as #root::ShaderType>::METADATA.uniform_min_alignment()
                {
                    let prev_offset = <Self as #root::ShaderType>::METADATA.offset(#i - 1);
                    let offset = <Self as #root::ShaderType>::METADATA.offset(#i);
                    let diff = offset - prev_offset;

                    let prev_size = <#prev_field_ty as #root::ShaderSize>::SHADER_SIZE.get();
                    let prev_size = min_alignment.round_up(prev_size);

                    #root::concat_assert!(
                        diff >= prev_size,
                        "offset between fields '", #prev_ident_name, "' and '", #name, "' must be at least ",
                        min_alignment.get(), " (currently: ", diff, ")"
                    )
                }
            }
        } else {
            quote! {()}
        };
        quote! {
            #ty_check,
            #field_offset_check,
            #field_offset_diff
        }
    });

    let alignments = field_data.iter().map(|data| data.alignment(root));

    let paddings = field_data.iter().enumerate().map(|(i, current)| {
        let is_first = i == 0;
        let is_last = i == field_data.len() - 1;

        let mut out = TokenStream::new();

        if !is_first {
            let prev_i = i - 1;

            let alignment = current.alignment(root);

            let extra_padding = field_data
                .get(prev_i)
                .and_then(|prev| prev.extra_padding(root))
                .map(|extra_padding| quote!(+ #extra_padding));

            out.extend(quote! {
                offsets[#i] = #alignment.round_up(offset);

                let padding = #alignment.padding_needed_for(offset);
                offset += padding;
                paddings[#prev_i] = padding #extra_padding;
            });
        };

        if is_last && is_runtime_sized {
            return out;
        }

        let size = current.size(root);
        out.extend(quote! {
            offset += #size;
        });

        if is_last {
            let extra_padding = current
                .extra_padding(root)
                .map(|extra_padding| quote!(+ #extra_padding));

            out.extend(quote! {
                paddings[#i] = struct_alignment.padding_needed_for(offset) #extra_padding;
            });
        }

        out
    });

    fn gen_body<'a>(
        field_data: &'a [FieldData],
        root: &'a Path,
        get_main: impl Fn(&Ident) -> TokenStream + 'a,
        get_padding: impl Fn(TokenStream) -> TokenStream + 'a,
    ) -> impl Iterator<Item = TokenStream> + 'a {
        field_data.iter().enumerate().map(move |(i, data)| {
            let ident = data.ident();

            let padding = {
                let i = Literal::usize_suffixed(i);
                quote! { <Self as #root::ShaderType>::METADATA.padding(#i) }
            };

            let main = get_main(ident);
            let padding = get_padding(padding);

            quote! {
                #main
                #padding
            }
        })
    }

    let write_into_buffer_body = gen_body(
        &field_data,
        root,
        |ident| {
            quote! {
                #root::WriteInto::write_into(&self.#ident, writer);
            }
        },
        |padding| {
            quote! {
                #root::Writer::advance(writer, #padding as ::core::primitive::usize);
            }
        },
    );

    let read_from_buffer_body = gen_body(
        &field_data,
        root,
        |ident| {
            quote! {
                #root::ReadFrom::read_from(&mut self.#ident, reader);
            }
        },
        |padding| {
            quote! {
                #root::Reader::advance(reader, #padding as ::core::primitive::usize);
            }
        },
    );

    let create_from_buffer_body = gen_body(
        &field_data,
        root,
        move |ident| {
            quote! {
                let #ident = #root::CreateFrom::create_from(reader);
            }
        },
        |padding| {
            quote! {
                #root::Reader::advance(reader, #padding as ::core::primitive::usize);
            }
        },
    );

    let field_idents = field_data.iter().map(|data| data.ident());
    let last_field = field_data.last().unwrap();
    let last_field_min_size = last_field.min_size(root);
    let last_field_ident = last_field.ident();

    let field_types = field_data.iter().map(|data| &data.field.ty);
    let field_types_2 = field_types.clone();
    let field_types_3 = field_types.clone();
    let field_types_4 = field_types.clone();
    let all_other = field_types.clone().take(last_field_index);
    let last_field_type = &last_field.field.ty;

    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let set_contained_rt_sized_array_length = if is_runtime_sized {
        quote! {
            writer.ctx.rts_array_length = ::core::option::Option::Some(
                #root::RuntimeSizedArray::len(&self.#last_field_ident)
                as ::core::primitive::u32
            );
        }
    } else {
        TokenStream::new()
    };

    let extra = match is_runtime_sized {
        true => quote! {
            impl #impl_generics #root::CalculateSizeFor for #name #ty_generics
            where
                Self: #root::ShaderType<ExtraMetadata = #root::StructMetadata<#nr_of_fields>>,
                #last_field_type: #root::CalculateSizeFor,
            {
                fn calculate_size_for(nr_of_el: ::core::primitive::u64) -> ::core::num::NonZeroU64 {
                    let mut offset = <Self as #root::ShaderType>::METADATA.last_offset();
                    offset += <#last_field_type as #root::CalculateSizeFor>::calculate_size_for(nr_of_el).get();
                    #root::SizeValue::new(<Self as #root::ShaderType>::METADATA.alignment().round_up(offset)).0
                }
            }
        },
        false => quote! {
            impl #impl_generics #root::ShaderSize for #name #ty_generics
            where
                #( #field_types: #root::ShaderSize, )*
            {}
        },
    };

    quote! {
        #( #field_trait_constraints )*

        #( #align_check )*

        #( #size_check )*

        impl #impl_generics #root::ShaderType for #name #ty_generics #where_clause
        where
            #( #all_other: #root::ShaderType + #root::ShaderSize, )*
            #last_field_type: #root::ShaderType,
        {
            type ExtraMetadata = #root::StructMetadata<#nr_of_fields>;
            const METADATA: #root::Metadata<Self::ExtraMetadata> = {
                let struct_alignment = #root::AlignmentValue::max([ #( #alignments, )* ]);

                let extra = {
                    let mut paddings = [0; #nr_of_fields];
                    let mut offsets = [0; #nr_of_fields];
                    let mut offset = 0;
                    #( #paddings )*
                    #root::StructMetadata { offsets, paddings }
                };

                let min_size = {
                    let mut offset = extra.offsets[#nr_of_fields - 1];
                    offset += #last_field_min_size;
                    #root::SizeValue::new(struct_alignment.round_up(offset))
                };

                #root::Metadata {
                    alignment: struct_alignment,
                    has_uniform_min_alignment: true,
                    min_size,
                    extra,
                }
            };

            const UNIFORM_COMPAT_ASSERT: fn() = || #root::consume_zsts([
                #( #uniform_check, )*
            ]);

            fn size(&self) -> ::core::num::NonZeroU64 {
                let mut offset = Self::METADATA.last_offset();
                offset += #root::ShaderType::size(&self.#last_field_ident).get();
                #root::SizeValue::new(Self::METADATA.alignment().round_up(offset)).0
            }
        }

        impl #impl_generics #root::WriteInto for #name #ty_generics
        where
            Self: #root::ShaderType<ExtraMetadata = #root::StructMetadata<#nr_of_fields>>,
            #( #field_types_2: #root::WriteInto, )*
        {
            fn write_into<B: #root::BufferMut>(&self, writer: &mut #root::Writer<B>) {
                #set_contained_rt_sized_array_length
                #( #write_into_buffer_body )*
            }
        }

        impl #impl_generics #root::ReadFrom for #name #ty_generics
        where
            Self: #root::ShaderType<ExtraMetadata = #root::StructMetadata<#nr_of_fields>>,
            #( #field_types_3: #root::ReadFrom, )*
        {
            fn read_from<B: #root::BufferRef>(&mut self, reader: &mut #root::Reader<B>) {
                #( #read_from_buffer_body )*
            }
        }

        impl #impl_generics #root::CreateFrom for #name #ty_generics
        where
            Self: #root::ShaderType<ExtraMetadata = #root::StructMetadata<#nr_of_fields>>,
            #( #field_types_4: #root::CreateFrom, )*
        {
            fn create_from<B: #root::BufferRef>(reader: &mut #root::Reader<B>) -> Self {
                #( #create_from_buffer_body )*

                #root::build_struct!(Self, #( #field_idents ),*)
            }
        }

        #extra
    }
}

fn generate_field_trait_constraints<'a>(
    input: &'a DeriveInput,
    field_data: &'a [FieldData],
    trait_for_last_field: TokenStream,
    trait_for_all_other_fields: TokenStream,
) -> impl Iterator<Item = TokenStream> + 'a {
    let (impl_generics, _, where_clause) = input.generics.split_for_impl();
    field_data.iter().enumerate().map(move |(i, data)| {
        let ty = &data.field.ty;

        let t = if i == field_data.len() - 1 {
            &trait_for_last_field
        } else {
            &trait_for_all_other_fields
        };

        quote_spanned! {ty.span()=>
            const _: fn() = || {
                #[allow(clippy::extra_unused_lifetimes, clippy::missing_const_for_fn)]
                fn check #impl_generics () #where_clause {
                    fn assert_impl<T: ?::core::marker::Sized + #t>() {}
                    assert_impl::<#ty>();
                }
            };
        }
    })
}
