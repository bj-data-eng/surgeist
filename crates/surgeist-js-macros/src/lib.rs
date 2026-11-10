//! Macros for the JavaScript engine.
#![forbid(unsafe_code)]
#![cfg_attr(not(test), forbid(clippy::unwrap_used))]

use crate::utils::RenameScheme;
use cow_utils::CowUtils;
use proc_macro::TokenStream;
use proc_macro2::{Literal, TokenTree};
use quote::{ToTokens, quote};
use std::collections::HashSet;
use syn::{
    Data, DeriveInput, Expr, ExprLit, Fields, FieldsNamed, Ident, Lit, LitStr, Token,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
};
use synstructure::{AddBounds, Structure, decl_derive};

mod embedded_module_loader;

mod class;
mod module;
mod utils;
mod value;

/// Create a JavaScript value from a JSON-like literal.
///
/// Array and object literals take a `ContextAccess` expression and return
/// `JsResult<JsValue>`. Pass `owner.access()` or `access.reborrow()`. The access
/// expression is evaluated once before the literal's fields or elements.
/// They reject object values from another runtime. Other inputs return `JsValue`.
/// Embedded Rust expressions retain caller control flow and cannot access the
/// macro's private capability; prepare same-context engine values before lending
/// access to the literal.
#[proc_macro]
pub fn js_value(input: TokenStream) -> TokenStream {
    value::js_value_impl(proc_macro2::TokenStream::from(input)).into()
}

/// Create a JavaScript object from a JSON-like literal.
///
/// The macro takes a `ContextAccess` expression and returns `JsResult<JsObject>`.
/// Pass `owner.access()` or `access.reborrow()`; the expression is evaluated once.
/// It rejects object values from another runtime. Embedded Rust expressions keep
/// their caller bindings rather than receiving a mutable context owner alias.
#[proc_macro]
pub fn js_object(input: TokenStream) -> TokenStream {
    value::js_object_impl(proc_macro2::TokenStream::from(input)).into()
}

/// Implementation of the inner iterator of the `embed_module!` macro. All
/// arguments are required.
///
/// # Warning
/// This should not be used directly as is, and instead should be used through
/// the `embed_module!` macro in `surgeist_js` for convenience.
#[proc_macro]
pub fn embed_module_inner(input: TokenStream) -> TokenStream {
    embedded_module_loader::embed_module_impl(input)
}

/// `boa_class` proc macro attribute that applies to an `impl XYZ` block and
/// add a `[surgeist_js::JsClass]` implementation for it.
///
/// It will transform functions in the `impl ...` block as follow (by default, see
/// below):
/// 1. `fn some_method(&self, ...) -> ... {}` will be added as class methods with
///    the name `some_method`, borrowing the object for the ref. This is dangerous
///    if the function execute/eval JavaScript back (potentially leading to a
///    `BorrowError`).
/// 2. `fn some_method(&mut self, ...) -> ... {}` will be added as class methods,
///    similar to the above but borrowing as mutable at runtime.
/// 3. `fn some_method(...) -> ... {}` (no self mention) will be added as a
///    static method.
/// 4. `#[boa(constructor)] fn ...(...) -> Self {}` (or returning `JsResult<Self>`)
///    will be used as the constructor of the class. If no constructor is declared,
///    `Default::default()` will be used instead. If the `Default` trait is not
///    defined for the type, an error will happen.
/// 5. `#[boa(getter)]`
///
/// To change this behaviour, you can use the following attributes on the function
/// declarations:
/// 1. `#[boa(rename = "...")]` renames the function in JavaScript with the string.
/// 2. `#[boa(getter)]` will declare a getter accessor.
/// 2. `#[boa(setter)]` will declare a setter accessor.
/// 3. `#[boa(static)]` will declare a static method.
/// 4. `#[boa(method)]` will declare a method.
/// 5. `#[boa(constructor)]` will declare a constructor.
/// 6. `#[boa(length = 123)]` sets the length of the function in JavaScript (ie. its
///    number of arguments accepted).
///
/// Multiple of those attributes can be added to a single method.
///
/// The top level `boa_class` supports the following:
/// 1. `#[boa_class(rename = "...")]` sets the name of the class in JavaScript.
/// 2. `#[boa(rename_all = "camelCase")]` will change the naming scheme of verbatim
///    to using "camelCase" or "none".
///
/// # Warning
/// This should not be used directly as is, and instead should be used through
/// the `embed_module!` macro in `surgeist_js` for convenience.
#[proc_macro_attribute]
pub fn boa_class(attr: TokenStream, item: TokenStream) -> TokenStream {
    class::class_impl(attr, item)
}

/// `boa_module` proc macro attribute for declaring a `surgeist_js::Module` based
/// on a Rust module. The original Rust module will also be exposed as is.
///
/// This macro exports two functions out of the existing module (and those
/// functions must not exist in the declared module):
///
/// ## `boa_module(realm: Option<Realm>, context: ContextAccess<'_>) -> JsResult<Module>`
/// Create a JavaScript module from the rust module's content.
///
/// ## `boa_register(realm: Option<Realm>, context: ContextAccess<'_>) -> JsResult<()>`
/// Register the constants, classes and functions from the module in the global
/// scope of the Realm (if specified) or the context (if no realm).
#[proc_macro_attribute]
pub fn boa_module(attr: TokenStream, item: TokenStream) -> TokenStream {
    module::module_impl(attr, item)
}

struct Static {
    literal: LitStr,
    ident: Ident,
}

impl Parse for Static {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let expr = Expr::parse(input)?;
        match expr {
            Expr::Tuple(expr) => {
                let mut elems = expr.elems.iter().cloned();
                let literal = elems
                    .next()
                    .ok_or_else(|| syn::Error::new_spanned(&expr, "invalid empty tuple"))?;
                let ident = elems.next();
                if elems.next().is_some() {
                    return Err(syn::Error::new_spanned(
                        &expr,
                        "invalid tuple with more than two elements",
                    ));
                }
                let Expr::Lit(ExprLit {
                    lit: Lit::Str(literal),
                    ..
                }) = literal
                else {
                    return Err(syn::Error::new_spanned(
                        literal,
                        "expected an UTF-8 string literal",
                    ));
                };

                let ident = if let Some(ident) = ident {
                    syn::parse2::<Ident>(ident.into_token_stream())?
                } else {
                    Ident::new(&literal.value().cow_to_uppercase(), literal.span())
                };

                Ok(Self { literal, ident })
            }
            Expr::Lit(expr) => match expr.lit {
                Lit::Str(str) => Ok(Self {
                    ident: Ident::new(&str.value().cow_to_uppercase(), str.span()),
                    literal: str,
                }),
                _ => Err(syn::Error::new_spanned(
                    expr,
                    "expected an UTF-8 string literal",
                )),
            },
            _ => Err(syn::Error::new_spanned(
                expr,
                "expected a string literal or a tuple expression",
            )),
        }
    }
}

struct Syms(Vec<Static>);

impl Parse for Syms {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let parsed = Punctuated::<Static, Token![,]>::parse_terminated(input)?;
        let mut spellings = HashSet::new();
        for symbol in &parsed {
            if !spellings.insert(symbol.literal.value()) {
                return Err(syn::Error::new_spanned(
                    &symbol.literal,
                    "duplicate static symbol spelling",
                ));
            }
        }
        Ok(Self(parsed.into_iter().collect()))
    }
}

#[doc(hidden)]
#[proc_macro]
pub fn static_syms(input: TokenStream) -> TokenStream {
    let literals = parse_macro_input!(input as Syms).0;

    let consts = literals.iter().enumerate().map(|(idx, lit)| {
        let doc = format!(
            "Symbol for the \"{}\" string.",
            lit.literal
                .value()
                .cow_replace('<', r"\<")
                .cow_replace('>', r"\>")
                .cow_replace('*', r"\*")
        );
        let ident = &lit.ident;
        let id = idx + 1;
        quote! {
            #[doc = #doc]
            pub const #ident: Self = Self::new(#id).expect("common symbol ID must be nonzero");
        }
    });

    let literals = literals.iter().map(|lit| &lit.literal).collect::<Vec<_>>();

    quote! {
        impl Sym {
            #(#consts)*
        }

        /// Common UTF-8 spellings in symbol declaration order.
        pub(super) const COMMON_STRINGS_UTF8: &[&str] = &[#(#literals),*];

        /// Common UTF-16 spellings in symbol declaration order.
        pub(super) const COMMON_STRINGS_UTF16: &[&[u16]] =
            &[#(::surgeist_js_macros::utf16!(#literals)),*];

        const _: () = assert!(COMMON_STRINGS_UTF8.len() < usize::MAX);
    }
    .into()
}

/// Construct a utf-16 array literal from a utf-8 [`str`] literal.
#[proc_macro]
pub fn utf16(input: TokenStream) -> TokenStream {
    let literal = parse_macro_input!(input as LitStr);
    let utf8 = literal.value();
    let utf16 = utf8.encode_utf16().collect::<Vec<_>>();
    quote! {
        [#(#utf16),*].as_slice()
    }
    .into()
}

/// Convert a utf8 string literal to a `JsString`
#[proc_macro]
pub fn js_str(input: TokenStream) -> TokenStream {
    let literal = parse_macro_input!(input as LitStr);

    let mut is_latin1 = true;
    let codepoints = literal
        .value()
        .encode_utf16()
        .map(|x| {
            if x > u8::MAX.into() {
                is_latin1 = false;
            }
            Literal::u16_unsuffixed(x)
        })
        .collect::<Vec<_>>();
    if is_latin1 {
        quote! {
            ::surgeist_js::string::JsStr::latin1([#(#codepoints),*].as_slice())
        }
    } else {
        quote! {
            ::surgeist_js::string::JsStr::utf16([#(#codepoints),*].as_slice())
        }
    }
    .into()
}

decl_derive! {
    [HeapTrace, attributes(heap_trace)] =>
    /// Derive the runtime heap's safe edge visitor.
    derive_heap_trace
}

fn derive_heap_trace(mut structure: Structure<'_>) -> proc_macro2::TokenStream {
    let outer_attributes = structure.ast().attrs.iter().chain(
        structure
            .variants()
            .iter()
            .flat_map(|variant| variant.ast().attrs),
    );
    for attribute in outer_attributes {
        if attribute.path().is_ident("heap_trace") {
            return syn::Error::new_spanned(attribute, "heap_trace applies only to fields")
                .into_compile_error();
        }
    }
    for variant in structure.variants() {
        for binding in variant.bindings() {
            for attribute in &binding.ast().attrs {
                if attribute.path().is_ident("heap_trace") {
                    match attribute.parse_args::<Ident>() {
                        Ok(argument) if argument == "skip" => {}
                        Ok(argument) => {
                            return syn::Error::new_spanned(argument, "expected `skip`")
                                .into_compile_error();
                        }
                        Err(error) => return error.into_compile_error(),
                    }
                }
            }
        }
    }
    structure.filter(|binding| {
        !binding
            .ast()
            .attrs
            .iter()
            .any(|attribute| attribute.path().is_ident("heap_trace"))
    });
    structure.add_bounds(AddBounds::Fields);
    // synstructure infers field bounds from type parameters. Const-only
    // field types need the same conditional bound.
    let const_parameters: Vec<_> = structure
        .ast()
        .generics
        .const_params()
        .map(|parameter| &parameter.ident)
        .collect();
    if !const_parameters.is_empty() {
        let predicates: Vec<syn::WherePredicate> = structure
            .variants()
            .iter()
            .flat_map(|variant| variant.bindings())
            .filter(|binding| binding.referenced_ty_params().is_empty())
            .map(|binding| &binding.ast().ty)
            .filter(|ty| mentions_const_parameter(ty.to_token_stream(), &const_parameters))
            .map(|ty| syn::parse_quote!(#ty: ::surgeist_js_gc::heap::Trace))
            .collect();
        for predicate in predicates {
            structure.add_where_predicate(predicate);
        }
    }
    structure.add_where_predicate(syn::parse_quote!(Self: 'static));
    let body =
        structure.each(|binding| quote!(::surgeist_js_gc::heap::Trace::trace(#binding, edges)?));
    structure.bound_impl(
        quote!(::surgeist_js_gc::heap::Trace),
        quote! {
            fn trace(
                &self,
                edges: &mut ::surgeist_js_gc::heap::EdgeVisitor<'_>,
            ) -> ::core::result::Result<(), ::surgeist_js_gc::heap::AccessError> {
                match *self { #body }
                ::core::result::Result::Ok(())
            }
        },
    )
}

fn mentions_const_parameter(tokens: proc_macro2::TokenStream, parameters: &[&Ident]) -> bool {
    tokens.into_iter().any(|token| match token {
        TokenTree::Ident(ident) => parameters.contains(&&ident),
        TokenTree::Group(group) => mentions_const_parameter(group.stream(), parameters),
        _ => false,
    })
}

decl_derive! {
    [JsData] =>
    /// Derive the `JsData` trait.
    derive_js_data
}

/// Derives the `JsData` trait.
#[allow(clippy::needless_pass_by_value)]
fn derive_js_data(s: Structure<'_>) -> proc_macro2::TokenStream {
    s.unbound_impl(quote!(::surgeist_js::JsData), quote!())
}

/// Derives the `TryFromJs` trait, with the `#[boa()]` attribute.
///
/// # Panics
///
/// It will panic if the user tries to derive the `TryFromJs` trait in an `enum` or a tuple struct.
#[proc_macro_derive(TryFromJs, attributes(boa))]
pub fn derive_try_from_js(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    let Data::Struct(data) = input.data else {
        panic!("you can only derive TryFromJs for structs");
    };

    let Fields::Named(fields) = data.fields else {
        panic!("you can only derive TryFromJs for named-field structs")
    };

    let renaming = match RenameScheme::from_named_attrs(&mut input.attrs.clone(), "rename_all") {
        Ok(renaming) => renaming.unwrap_or(RenameScheme::None),
        Err((span, msg)) => return syn::Error::new(span, msg).to_compile_error().into(),
    };

    let context_binding = if fields.named.is_empty() {
        quote! { _context }
    } else {
        quote! { mut context }
    };
    let conv = generate_conversion(fields, renaming).unwrap_or_else(to_compile_errors);

    let type_name = input.ident;

    // Build the output, possibly using quasi-quotation
    let expanded = quote! {
        impl ::surgeist_js::value::TryFromJs for #type_name {
            fn try_from_js(value: &surgeist_js::JsValue, #context_binding: surgeist_js::ContextAccess<'_>)
                -> surgeist_js::JsResult<Self> {
                let o = value.as_object().ok_or_else(|| ::surgeist_js::JsError::from(
                    ::surgeist_js::JsNativeError::typ()
                        .with_message("value is not an object")
                ))?;
                #conv
            }
        }
    };

    // Hand the output tokens back to the compiler
    expanded.into()
}

/// Generates the conversion field by field.
fn generate_conversion(
    fields: FieldsNamed,
    rename: RenameScheme,
) -> Result<proc_macro2::TokenStream, Vec<syn::Error>> {
    use syn::spanned::Spanned;

    let mut field_list = Vec::with_capacity(fields.named.len());
    let mut final_fields = Vec::with_capacity(fields.named.len());

    for field in fields.named {
        let span = field.span();
        let name = field.ident.ok_or_else(|| {
            vec![syn::Error::new(
                span,
                "you can only derive `TryFromJs` for named-field structs",
            )]
        })?;

        field_list.push(name.clone());

        let mut from_js_with = None;
        let mut field_name = rename.rename(format!("{name}"));
        for attr in field
            .attrs
            .into_iter()
            .filter(|attr| attr.path().is_ident("boa"))
        {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("from_js_with") {
                    let value = meta.value()?;
                    from_js_with = Some(value.parse::<LitStr>()?);
                    Ok(())
                } else if meta.path.is_ident("rename") {
                    let value = meta.value()?;
                    field_name = value.parse::<LitStr>()?.value();
                    Ok(())
                } else if meta.path.is_ident("into_js_with") {
                    let _unused = meta.value()?.parse::<LitStr>()?;
                    Ok(())
                } else if meta.path.is_ident("skip") && meta.input.is_empty() {
                    Ok(())
                } else {
                    Err(meta.error(
                        "invalid syntax in the `#[boa()]` attribute. \
                               Note that this attribute only accepts the following syntax: \
                            \n* `#[boa(from_js_with = \"fully::qualified::path\")]` \
                            \n* `#[boa(rename = \"jsPropertyName\")]` \
                            ",
                    ))
                }
            })
            .map_err(|err| vec![err])?;
        }

        let error_str = format!("cannot get property {name} of value");
        final_fields.push(quote! {
            let #name = match props.get(&::surgeist_js::js_string!(#field_name).into())? {
                Some(pd) => pd.value().ok_or_else(|| ::surgeist_js::JsError::from(
                        ::surgeist_js::JsNativeError::typ().with_message(#error_str)
                    ))?.clone().try_js_into(context.reborrow())?,
                None => ::surgeist_js::JsValue::undefined().try_js_into(context.reborrow())?,
            };
        });

        if let Some(method) = from_js_with {
            let ident = Ident::new(&method.value(), method.span());
            final_fields.push(quote! {
                let #name = #ident(&#name, context.reborrow())?;
            });
        }
    }

    // TODO: this could possibly skip accessors. Consider using `JsObject::get` instead.
    Ok(quote! {
        let o = o.borrow()?;
        let props = o.properties();
        #(#final_fields)*
        Ok(Self {
            #(#field_list),*
        })
    })
}

/// Generates a list of compile errors.
#[allow(clippy::needless_pass_by_value)]
fn to_compile_errors(errors: Vec<syn::Error>) -> proc_macro2::TokenStream {
    let compile_errors = errors.iter().map(syn::Error::to_compile_error);
    quote!(#(#compile_errors)*)
}

/// Derives the `TryIntoJs` trait, with the `#[boa()]` attribute.
///
/// # Panics
///
/// It will panic if the user tries to derive the `TryIntoJs` trait in an `enum` or a tuple struct.
#[proc_macro_derive(TryIntoJs, attributes(boa))]
pub fn derive_try_into_js(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    let Data::Struct(data) = input.data else {
        panic!("you can only derive TryFromJs for structs");
    };
    // TODO: Enums ?

    let Fields::Named(fields) = data.fields else {
        panic!("you can only derive TryFromJs for named-field structs")
    };

    let renaming = match RenameScheme::from_named_attrs(&mut input.attrs.clone(), "rename_all") {
        Ok(renaming) => renaming.unwrap_or(RenameScheme::None),
        Err((span, msg)) => return syn::Error::new(span, msg).to_compile_error().into(),
    };

    let props = generate_obj_properties(fields, renaming)
        .map_err(|err| vec![err])
        .unwrap_or_else(to_compile_errors);
    let context_binding = if props.is_empty() {
        quote! { context }
    } else {
        quote! { mut context }
    };

    let type_name = input.ident;

    // Build the output, possibly using quasi-quotation
    let expanded = quote! {
        impl ::surgeist_js::value::TryIntoJs for #type_name {
            fn try_into_js(&self, #context_binding: surgeist_js::ContextAccess<'_>) -> surgeist_js::JsResult<surgeist_js::JsValue> {
                let obj = surgeist_js::JsObject::default(context.intrinsics())?;
                #props
                surgeist_js::JsResult::Ok(obj.into())
            }
        }
    };

    // Hand the output tokens back to the compiler
    expanded.into()
}

/// Generates property creation for object.
fn generate_obj_properties(
    fields: FieldsNamed,
    renaming: RenameScheme,
) -> Result<proc_macro2::TokenStream, syn::Error> {
    use syn::spanned::Spanned;

    let mut prop_ctors = Vec::with_capacity(fields.named.len());

    for field in fields.named {
        let span = field.span();
        let name = field.ident.ok_or_else(|| {
            syn::Error::new(
                span,
                "you can only derive `TryIntoJs` for named-field structs",
            )
        })?;

        let mut into_js_with = None;
        let mut prop_key = renaming.rename(format!("{name}"));
        let mut skip = false;

        for attr in field
            .attrs
            .into_iter()
            .filter(|attr| attr.path().is_ident("boa"))
        {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("into_js_with") {
                    let value = meta.value()?;
                    into_js_with = Some(value.parse::<LitStr>()?);
                    Ok(())
                } else if meta.path.is_ident("rename") {
                    let value = meta.value()?;
                    prop_key = value.parse::<LitStr>()?.value();
                    Ok(())
                } else if meta.path.is_ident("skip") && meta.input.is_empty() {
                    skip = true;
                    Ok(())
                } else if meta.path.is_ident("from_js_with") {
                    let _unused = meta.value()?.parse::<LitStr>()?;
                    Ok(())
                } else {
                    Err(meta.error(
                        "invalid syntax in the `#[boa()]` attribute. \
                               Note that this attribute only accepts the following syntax: \
                            \n* `#[boa(into_js_with = \"fully::qualified::path\")]` \
                            \n* `#[boa(rename = \"jsPropertyName\")]` \
                            \n* `#[boa(skip)]` \
                            ",
                    ))
                }
            })?;
        }

        if skip {
            continue;
        }

        let value = if let Some(into_js_with) = into_js_with {
            let into_js_with = Ident::new(&into_js_with.value(), into_js_with.span());
            quote! { #into_js_with(&self.#name, context.reborrow())? }
        } else {
            quote! { surgeist_js::value::TryIntoJs::try_into_js(&self.#name, context.reborrow())? }
        };
        prop_ctors.push(quote! {
            obj.create_data_property_or_throw(surgeist_js::js_string!(#prop_key), #value, context.reborrow())?;
        });
    }

    Ok(quote! { #(#prop_ctors)* })
}
