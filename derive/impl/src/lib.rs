mod codegen;

use proc_macro2::{Ident, Literal, Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    parse_quote,
    punctuated::Punctuated,
    spanned::Spanned,
    token::Comma,
    Data, DataStruct, DeriveInput, Error, Fields, FieldsNamed, GenericParam, LitInt, Path, Token,
    Type,
};

pub use syn;

use crate::codegen::{
    gen_create_from_impl, gen_field_trait_constraints, gen_read_from_impl, gen_shader_size_impl,
    gen_shader_type_impl, gen_write_into_impl,
};

#[macro_export]
macro_rules! implement {
    ($path:expr) => {
        #[proc_macro_derive(ShaderType, attributes(shader))]
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

#[derive(Debug)]
pub struct AlignmentAttr(u32);

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

#[derive(Debug)]
pub struct StaticSizeAttr(u32);

impl Parse for StaticSizeAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let span = input.span();
        match input
            .parse::<LitInt>()
            .and_then(|lit| lit.base10_parse::<u32>())
        {
            Ok(num) => Ok(Self(num)),
            _ => Err(syn::Error::new(span, "expected u32 literal")),
        }
    }
}

#[derive(Debug)]
pub enum SizeAttr {
    Static(StaticSizeAttr),
    Runtime,
}

impl Parse for SizeAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let span = input.span();
        match input.parse::<StaticSizeAttr>() {
            Ok(static_size) => Ok(SizeAttr::Static(static_size)),
            _ => match input.parse::<Path>() {
                Ok(ident) if ident.is_ident("runtime") => Ok(SizeAttr::Runtime),
                _ => Err(syn::Error::new(
                    span,
                    "expected u32 literal or `runtime` identifier",
                )),
            },
        }
    }
}

#[derive(Debug)]
pub enum ShaderAttr {
    Align { attr: AlignmentAttr, span: Span },
    Size { attr: SizeAttr, span: Span },
}

impl Parse for ShaderAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident_span = input.span();
        let Ok(ident) = input.parse::<Ident>() else {
            return Err(syn::Error::new(ident_span, "expected `align` or `size`"));
        };

        match ident.to_string().as_str() {
            "align" => {
                if !input.peek(syn::token::Paren) {
                    return Err(syn::Error::new(
                        ident_span,
                        "expected attribute arguments in parentheses: `align(...)`",
                    ));
                }

                let args;
                parenthesized!(args in input);
                let attr_span = args.span();
                let align_attr: AlignmentAttr = args.parse()?;
                Ok(ShaderAttr::Align {
                    attr: align_attr,
                    span: attr_span,
                })
            }
            "size" => {
                if !input.peek(syn::token::Paren) {
                    return Err(syn::Error::new(
                        ident_span,
                        "expected attribute arguments in parentheses: `size(...)`",
                    ));
                }

                let args;
                parenthesized!(args in input);
                let attr_span = args.span();
                let size_attr: SizeAttr = args.parse()?;
                Ok(ShaderAttr::Size {
                    attr: size_attr,
                    span: attr_span,
                })
            }
            _ => Err(syn::Error::new(
                ident_span,
                "unknown shader attribute, expected `align` or `size`",
            )),
        }
    }
}

#[derive(Debug)]
pub struct ShaderAttrList(Punctuated<ShaderAttr, Token![,]>);
impl Parse for ShaderAttrList {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self(input.parse_terminated(ShaderAttr::parse, Token![,])?))
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
                if !(attr.meta.path().is_ident("shader")) {
                    continue;
                }

                let shader_attrs = match attr.parse_args::<ShaderAttrList>() {
                    Ok(attrs) => attrs,
                    Err(err) => {
                        errors.append(err);
                        continue;
                    }
                };

                for shader_attr in shader_attrs.0 {
                    match shader_attr {
                        ShaderAttr::Align { attr, span } => {
                            if data.align.is_some() {
                                let err = syn::Error::new(span, "duplicate `align(X)` attribute");
                                errors.append(err);
                                continue;
                            }

                            data.align = Some((attr.0, span));
                        }
                        ShaderAttr::Size { attr, span } => {
                            if data.size.is_some() || is_runtime_sized {
                                let err = syn::Error::new(span, "duplicate `size(X)` attribute");
                                errors.append(err);
                                continue;
                            }

                            match attr {
                                SizeAttr::Runtime => {
                                    if i == last_field_index {
                                        is_runtime_sized = true;
                                    } else {
                                        let err = syn::Error::new(
                                            span,
                                            "only the last field can be `size(runtime)`",
                                        );
                                        errors.append(err);
                                        continue;
                                    }
                                }
                                SizeAttr::Static(attr) => {
                                    data.size = Some((attr.0, span));
                                }
                            }
                        }
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
                                "`ArrayLength` type can only be used within a struct containing a runtime-sized array marked as `#[shader(size(runtime))]`!",
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

    let field_trait_constraints = gen_field_trait_constraints(
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
                                "shader(align) attribute value must be at least ", alignment, " (field's type alignment)"
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

    let shader_type_impl = gen_shader_type_impl(&input, &field_data, root, is_runtime_sized);

    let write_into_impl = gen_write_into_impl(&input, &field_data, root, is_runtime_sized);

    let read_from_impl = gen_read_from_impl(&input, &field_data, root);

    let create_from_impl = gen_create_from_impl(&input, &field_data, root);

    let extra = gen_shader_size_impl(&input, &field_data, root, is_runtime_sized);

    // Note:
    // The unused HRTBs on WriteInto, ReadFrom and CreateFrom are there
    // to avoid #![feature(trivial_bounds)].
    // Workaround found here: https://github.com/rust-lang/rust/issues/48214#issuecomment-1150463333

    quote! {
        #( #field_trait_constraints )*

        #( #align_check )*

        #( #size_check )*

        #shader_type_impl

        #write_into_impl

        #read_from_impl

        #create_from_impl

        #extra
    }
}
