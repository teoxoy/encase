use proc_macro2::{Ident, Literal, Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, parse_quote,
    punctuated::Punctuated,
    spanned::Spanned,
    token::Comma,
    Data, DataStruct, DeriveInput, Error, Fields, FieldsNamed, GenericParam, LitInt, Path, Type,
};

#[proc_macro_derive(WgslType, attributes(assert_uniform_compat, align, size))]
pub fn derive_wgsl_struct(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let expanded = emit(input);

    proc_macro::TokenStream::from(expanded)
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
                <#ty as #root::WgslType>::METADATA.alignment()
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
                <#ty as #root::Size>::SIZE.get()
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
                <#ty as #root::WgslType>::METADATA.min_size().get()
            }
        }
    }

    fn extra_padding(&self, root: &Path) -> Option<TokenStream> {
        self.size.as_ref().map(|(size, _)| {
            let size = Literal::u64_suffixed(*size as u64);
            let ty = &self.field.ty;
            let original_size = quote! { <#ty as #root::Size>::SIZE.get() };
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

fn emit(input: DeriveInput) -> TokenStream {
    let root: &Path = &parse_quote!(::encase::private);

    let fields = match get_named_struct_fields(&input.data) {
        Ok(fields) => fields,
        Err(e) => return e.into_compile_error(),
    };

    let assert_uniform_compat = input
        .attrs
        .iter()
        .any(|attr| attr.path.is_ident("assert_uniform_compat"));

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
            quote!(#root::WgslType + #root::RuntimeSizedArray)
        } else {
            quote!(#root::WgslType + #root::Size)
        },
        quote!(#root::WgslType + #root::Size),
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
                            let alignment = <#ty as #root::WgslType>::METADATA.alignment().get();
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
                            let size = <#ty as #root::Size>::SIZE.get();
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
            <#ty as #root::WgslType>::UNIFORM_COMPAT_ASSERT
        };
        let ident = data.ident();
        let name = ident.to_string();
        let field_check = quote_spanned! {ident.span()=> {
            let offset = <Self as #root::WgslType>::METADATA.offset(#i);
            #root::concat_assert!(
                #root::MIN_UNIFORM_ALIGNMENT.is_aligned(offset),
                "offset of field '", #name, "' must be a multiple of ", #root::MIN_UNIFORM_ALIGNMENT.get(),
                " (current offset: ", offset, ")"
            )
        }};
        quote! {
            #ty_check,
            #field_check
        }
    });

    let alignments = field_data.iter().map(|data| data.alignment(root));

    let paddings = field_data
        .iter()
        .take(last_field_index)
        .zip(field_data.iter().skip(1))
        .enumerate()
        .map(|(i, (prev, current))| {
            let is_first = i == 0;
            let is_last = i == last_field_index - 1;

            let alignment = current.alignment(root);
            let size = current.size(root);

            let curr_i = Literal::usize_suffixed(i);
            let curr_offset_i = Literal::usize_suffixed(i + 1);
            let mut padding = quote! {
                offsets[#curr_offset_i] = #alignment.round_up(offset);

                let padding = #alignment.padding_needed_for(offset);
                paddings[#curr_i] = padding;
            };
            let extra_padding_for_prev_field = prev.extra_padding(root);
            if let Some(extra_padding) = extra_padding_for_prev_field {
                padding.extend(quote! {
                    paddings[#curr_i] += #extra_padding;
                });
            }

            if is_first {
                let prev_size = prev.size(root);
                padding = quote! {
                    offset += #prev_size;
                    #padding
                }
            }

            let padding_add_to_offset = quote! {
                #padding
                offset += padding + #size;
            };

            if is_last {
                let last_i = Literal::usize_suffixed(i + 1);
                if is_runtime_sized {
                    quote! {
                        #padding
                        offset += padding;
                    }
                } else {
                    let mut base = quote! {
                        #padding_add_to_offset
                        paddings[#last_i] = struct_alignment.padding_needed_for(offset);
                    };

                    let extra_padding_for_curr_field = current.extra_padding(root);
                    if let Some(extra_padding) = extra_padding_for_curr_field {
                        base.extend(quote! {
                            paddings[#last_i] += #extra_padding;
                        });
                    }

                    base
                }
            } else {
                padding_add_to_offset
            }
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
                quote! { <Self as #root::WgslType>::METADATA.padding(#i) }
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

    let assert_uniform_compat = if assert_uniform_compat {
        quote! {
            const _: () = <#name as #root::WgslType>::UNIFORM_COMPAT_ASSERT;
        }
    } else {
        TokenStream::new()
    };

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
                Self: #root::WgslType<ExtraMetadata = #root::StructMetadata<#nr_of_fields>>,
                #last_field_type: #root::CalculateSizeFor,
            {
                fn calculate_size_for(nr_of_el: ::core::primitive::u64) -> ::core::num::NonZeroU64 {
                    let mut offset = <Self as #root::WgslType>::METADATA.last_offset();
                    offset += <#last_field_type as #root::CalculateSizeFor>::calculate_size_for(nr_of_el).get();
                    #root::SizeValue::new(<Self as #root::WgslType>::METADATA.alignment().round_up(offset)).0
                }
            }
        },
        false => quote! {
            impl #impl_generics #root::Size for #name #ty_generics
            where
                #( #field_types: #root::Size, )*
            {}
        },
    };

    quote! {
        #( #field_trait_constraints )*

        #( #align_check )*

        #( #size_check )*

        #assert_uniform_compat

        impl #impl_generics #root::WgslType for #name #ty_generics #where_clause
        where
            #( #all_other: #root::WgslType + #root::Size, )*
            #last_field_type: #root::WgslType,
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
                    min_size,
                    extra,
                }
            };

            const UNIFORM_COMPAT_ASSERT: () = #root::consume_zsts([
                #( #uniform_check, )*
            ]);

            fn size(&self) -> ::core::num::NonZeroU64 {
                let mut offset = Self::METADATA.last_offset();
                offset += #root::WgslType::size(&self.#last_field_ident).get();
                #root::SizeValue::new(Self::METADATA.alignment().round_up(offset)).0
            }
        }

        impl #impl_generics #root::WriteInto for #name #ty_generics
        where
            Self: #root::WgslType<ExtraMetadata = #root::StructMetadata<#nr_of_fields>>,
            #( #field_types_2: #root::WriteInto, )*
        {
            fn write_into<B: #root::BufferMut>(&self, writer: &mut #root::Writer<B>) {
                #set_contained_rt_sized_array_length
                #( #write_into_buffer_body )*
            }
        }

        impl #impl_generics #root::ReadFrom for #name #ty_generics
        where
            Self: #root::WgslType<ExtraMetadata = #root::StructMetadata<#nr_of_fields>>,
            #( #field_types_3: #root::ReadFrom, )*
        {
            fn read_from<B: #root::BufferRef>(&mut self, reader: &mut #root::Reader<B>) {
                #( #read_from_buffer_body )*
            }
        }

        impl #impl_generics #root::CreateFrom for #name #ty_generics
        where
            Self: #root::WgslType<ExtraMetadata = #root::StructMetadata<#nr_of_fields>>,
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
                #[allow(clippy::extra_unused_lifetimes)]
                fn check #impl_generics () #where_clause {
                    fn assert_impl<T: ?::core::marker::Sized + #t>() {}
                    assert_impl::<#ty>();
                }
            };
        }
    })
}
