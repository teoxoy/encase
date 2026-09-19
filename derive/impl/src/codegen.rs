use proc_macro2::{Literal, TokenStream};
use quote::{quote, quote_spanned, ToTokens};
use syn::spanned::Spanned;

use crate::FieldData;

struct FieldCtx<'a> {
    current: &'a FieldData,
    index: usize,
    next: Option<&'a FieldData>,
    prev: Option<&'a FieldData>,
}

fn gen_uniform_check(ctx: &FieldCtx, root: &syn::Path) -> TokenStream {
    let ty = &ctx.current.field.ty;
    let ty_check = quote_spanned! {ty.span()=>
        <#ty as #root::ShaderType>::UNIFORM_COMPAT_ASSERT()
    };

    let ident = ctx.current.ident();
    let i = ctx.index;
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

    let field_offset_diff = gen_field_offset_diff_check(ctx, root).unwrap_or(quote! {()});

    quote! {
        #ty_check,
        #field_offset_check,
        #field_offset_diff
    }
}

fn gen_field_offset_diff_check(ctx: &FieldCtx, root: &syn::Path) -> Option<TokenStream> {
    let ident = ctx.current.ident();
    let name = ident.to_string();
    let i = ctx.index;

    ctx.prev.as_ref().map(|prev| {
        let prev_field_ty = &prev.field.ty;
        let prev_ident_name = prev.ident().to_string();

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
    })
}

fn gen_padding(ctx: &FieldCtx, is_runtime_sized: bool, root: &syn::Path) -> TokenStream {
    let i = ctx.index;
    let mut out = TokenStream::new();

    if let Some(prev) = ctx.prev {
        let prev_i = ctx.index - 1;
        let alignment = ctx.current.alignment(root);

        let extra_padding = prev
            .extra_padding(root)
            .map(|extra_padding| quote!(+ #extra_padding));

        out.extend(quote! {
            offsets[#i] = #alignment.round_up(offset);

            let padding = #alignment.padding_needed_for(offset);
            offset += padding;
            paddings[#prev_i] = padding #extra_padding;
        });
    };

    if ctx.next.is_none() && is_runtime_sized {
        return out;
    }

    let size = ctx.current.size(root);
    out.extend(quote! {
        offset += #size;
    });

    if ctx.next.is_none() {
        let extra_padding = ctx
            .current
            .extra_padding(root)
            .map(|extra_padding| quote!(+ #extra_padding));

        out.extend(quote! {
            paddings[#i] = struct_alignment.padding_needed_for(offset) #extra_padding;
        });
    }

    out
}

fn gen_body(
    field_data: &[FieldData],
    root: &syn::Path,
    get_main: impl Fn(&syn::Ident) -> TokenStream,
    get_padding: impl Fn(TokenStream) -> TokenStream,
) -> Vec<TokenStream> {
    field_data
        .iter()
        .enumerate()
        .map(move |(i, data)| {
            let ident = &data.ident();

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
        .collect()
}

pub fn gen_shader_size_impl(
    input: &syn::DeriveInput,
    field_data: &[FieldData],
    root: &syn::Path,
    is_runtime_sized: bool,
) -> TokenStream {
    let last_field_type = &field_data.last().unwrap().field.ty;
    let field_types = field_data.iter().map(|data| &data.field.ty);
    let nr_of_fields = field_data.len();
    let name = &input.ident;
    let (impl_generics, ty_generics, _) = input.generics.split_for_impl();

    match is_runtime_sized {
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
    }
}

pub fn gen_field_trait_constraints<'a>(
    input: &'a syn::DeriveInput,
    field_data: &'a [FieldData],
    trait_for_last_field: TokenStream,
    trait_for_all_other_fields: TokenStream,
) -> impl Iterator<Item = TokenStream> + 'a {
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    field_data.iter().enumerate().map(move |(i, data)| {
        let ty = &data.field.ty;

        let t = if i == field_data.len() - 1 {
            &trait_for_last_field
        } else {
            &trait_for_all_other_fields
        };

        if ty_generics.to_token_stream().is_empty() {
            quote_spanned! {ty.span()=>
                const _: fn() = || {
                    #[allow(clippy::extra_unused_lifetimes, clippy::missing_const_for_fn, clippy::extra_unused_type_parameters)]
                    fn check #impl_generics () #where_clause {
                        fn assert_impl<T: ?::core::marker::Sized + #t>() {}
                        assert_impl::<#ty>();
                    }
                    check ();
                };
            }
        } else {
            // Case with type generics is not checked for now
            quote_spanned! {ty.span()=>
                const _: fn() = || {};
            }
        }
    })
}

pub fn gen_shader_type_impl(
    input: &syn::DeriveInput,
    field_data: &[FieldData],
    root: &syn::Path,
    is_runtime_sized: bool,
) -> TokenStream {
    let nr_of_fields = field_data.len();
    let contexts: Vec<_> = (0..nr_of_fields)
        .map(|i| FieldCtx {
            current: &field_data[i],
            index: i,
            next: (i + 1 < nr_of_fields).then(|| &field_data[i + 1]),
            prev: (i > 0).then(|| &field_data[i - 1]),
        })
        .collect();

    let uniform_checks: Vec<_> = contexts
        .iter()
        .map(|ctx| gen_uniform_check(ctx, root))
        .collect();

    let alignments: Vec<_> = field_data.iter().map(|data| data.alignment(root)).collect();

    let paddings: Vec<_> = contexts
        .iter()
        .map(|ctx| gen_padding(ctx, is_runtime_sized, root))
        .collect();

    let last_field = field_data.last().unwrap();
    let last_field_min_size = last_field.min_size(root);
    let last_field_ident = &last_field.ident();

    let field_types: Vec<_> = field_data.iter().map(|data| &data.field.ty).collect();
    let all_other = &field_types[..field_types.len() - 1];
    let last_field_type = &last_field.field.ty;

    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    quote! {

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
                    is_pod: false,
                    extra,
                }
            };

            const UNIFORM_COMPAT_ASSERT: fn() = || #root::consume_zsts([
                #( #uniform_checks, )*
            ]);

            fn size(&self) -> ::core::num::NonZeroU64 {
                let mut offset = Self::METADATA.last_offset();
                offset += #root::ShaderType::size(&self.#last_field_ident).get();
                #root::SizeValue::new(Self::METADATA.alignment().round_up(offset)).0
            }
        }

    }
}

pub fn gen_write_into_impl(
    input: &syn::DeriveInput,
    field_data: &[FieldData],
    root: &syn::Path,
    is_runtime_sized: bool,
) -> TokenStream {
    let last_field_ident = &field_data.last().unwrap().ident();

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

    let write_into_buffer_body = gen_body(
        field_data,
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

    let name = &input.ident;
    let (impl_generics, ty_generics, _) = input.generics.split_for_impl();
    let field_types = field_data.iter().map(|data| &data.field.ty);
    let nr_of_fields = field_data.len();
    quote! {
        impl #impl_generics #root::WriteInto for #name #ty_generics
        where
            Self: #root::ShaderType<ExtraMetadata = #root::StructMetadata<#nr_of_fields>>,
            #( for<'__> #field_types: #root::WriteInto, )*
        {
            #[inline]
            fn write_into<B: #root::BufferMut>(&self, writer: &mut #root::Writer<B>) {
                #set_contained_rt_sized_array_length
                #( #write_into_buffer_body )*
            }
        }
    }
}

pub fn gen_read_from_impl(
    input: &syn::DeriveInput,
    field_data: &[FieldData],
    root: &syn::Path,
) -> TokenStream {
    let read_from_buffer_body = gen_body(
        field_data,
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

    let name = &input.ident;
    let (impl_generics, ty_generics, _) = input.generics.split_for_impl();
    let field_types = field_data.iter().map(|data| &data.field.ty);
    let nr_of_fields = field_data.len();
    quote! {
        impl #impl_generics #root::ReadFrom for #name #ty_generics
        where
            Self: #root::ShaderType<ExtraMetadata = #root::StructMetadata<#nr_of_fields>>,
            #( for<'__> #field_types: #root::ReadFrom, )*
        {
            #[inline]
            fn read_from<B: #root::BufferRef>(&mut self, reader: &mut #root::Reader<B>) {
                #( #read_from_buffer_body )*
            }
        }

    }
}

pub fn gen_create_from_impl(
    input: &syn::DeriveInput,
    field_data: &[FieldData],
    root: &syn::Path,
) -> TokenStream {
    let create_from_buffer_body = gen_body(
        field_data,
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

    let name = &input.ident;
    let (impl_generics, ty_generics, _) = input.generics.split_for_impl();
    let field_types: Vec<_> = field_data.iter().map(|data| &data.field.ty).collect();
    let nr_of_fields = field_data.len();
    let field_idents = field_data.iter().map(|f| &f.field.ident);

    quote! {
        impl #impl_generics #root::CreateFrom for #name #ty_generics
        where
            Self: #root::ShaderType<ExtraMetadata = #root::StructMetadata<#nr_of_fields>>,
            #( for<'__> #field_types: #root::CreateFrom, )*
        {
            #[inline]
            fn create_from<B: #root::BufferRef>(reader: &mut #root::Reader<B>) -> Self {
                #( #create_from_buffer_body )*

                #root::build_struct!(Self, #( #field_idents ),*)
            }
        }
    }
}
