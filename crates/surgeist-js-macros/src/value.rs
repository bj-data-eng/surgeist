use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::{Brace, Bracket};
use syn::{Expr, Ident, LitStr, Token, braced, bracketed, parse2};

/// The key can be an identifier (which will be stringified), or an actual string
/// literal.
enum Key {
    Bracketed(Expr),
    Ident(Ident),
    StringLiteral(LitStr),
}

impl Parse for Key {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        if input.peek(Bracket) {
            let content;
            let _bracket = bracketed!(content in input);
            Ok(Key::Bracketed(content.parse()?))
        } else if input.peek(Ident) {
            Ok(Key::Ident(input.parse()?))
        } else if input.peek(LitStr) {
            Ok(Key::StringLiteral(input.parse()?))
        } else {
            Err(input.error("Expected a field name"))
        }
    }
}

/// A value, which itself can recursively be an object, an array, a literal or
/// an expression.
enum Value {
    Object(Object),
    Array(Array),
    String(LitStr),
    Expr(Expr),
}

enum ExpandedValue {
    Plain(TokenStream),
    Checked(TokenStream),
}

impl ExpandedValue {
    fn tokens(self) -> TokenStream {
        match self {
            Self::Plain(tokens) | Self::Checked(tokens) => tokens,
        }
    }

    fn bind(self, name: &Ident, setup: &TokenStream, body: TokenStream) -> TokenStream {
        match self {
            Self::Plain(value) => quote! {
                let #name = { #setup #value };
                #body
            },
            // A match propagates a nested literal error without changing where control
            // flow in an embedded Rust expression returns or breaks.
            Self::Checked(value) => quote! {
                match { #setup #value } {
                    ::core::result::Result::Ok(#name) => { #body }
                    ::core::result::Result::Err(error) => ::core::result::Result::Err(error),
                }
            },
        }
    }
}

impl Value {
    fn needs_context(&self) -> bool {
        matches!(self, Self::Object(_) | Self::Array(_))
    }

    fn output(&self, context: Option<&Ident>) -> syn::Result<ExpandedValue> {
        let container = match self {
            Value::Object(object) => object.output(context)?,
            Value::Array(array) => array.output(context)?,
            Value::String(str) => {
                return Ok(ExpandedValue::Plain(quote! {
                    ::surgeist_js::JsValue::from( ::surgeist_js::js_str!( #str ) )
                }));
            }
            Value::Expr(e) => {
                return Ok(ExpandedValue::Plain(quote! {
                    ::surgeist_js::JsValue::from( #e )
                }));
            }
        };
        Ok(ExpandedValue::Checked(quote! {
            #container.map(::surgeist_js::JsValue::from)
        }))
    }
}

impl Parse for Value {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        if input.peek(Brace) {
            Ok(Self::Object(input.parse()?))
        } else if input.peek(Bracket) {
            Ok(Self::Array(input.parse()?))
        } else if input.peek(LitStr) {
            Ok(Self::String(input.parse()?))
        } else {
            Ok(Self::Expr(input.parse()?))
        }
    }
}

/// An object is built of multiple key-value pairs.
struct KeyValue {
    key: Key,
    _colon: Token![:],
    value: Value,
}

impl Parse for KeyValue {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        Ok(Self {
            key: input.parse()?,
            _colon: input.parse()?,
            value: input.parse()?,
        })
    }
}

/// An object declaration.
struct Object {
    _brace: Brace,
    fields: Punctuated<KeyValue, Token![,]>,
}

impl Object {
    fn output(&self, context: Option<&Ident>) -> syn::Result<TokenStream> {
        let Some(c_ident) = context else {
            return Err(syn::Error::new(
                Span::call_site(),
                "Need to specify a ContextAccess expression.",
            ));
        };

        let object = Ident::new("__surgeist_js_object", Span::mixed_site());
        let property_value = Ident::new("__surgeist_js_property_value", Span::mixed_site());
        let field_context = Ident::new("__surgeist_js_field_context", Span::mixed_site());
        let mut body = quote! { #object.build() };
        for field in self.fields.iter().rev() {
            let key = match &field.key {
                Key::Bracketed(expr) => {
                    quote! { ::surgeist_js::property::PropertyKey::from( #expr ) }
                }
                Key::Ident(ident) => {
                    let ident = ident.to_string();
                    quote! { ::surgeist_js::js_string!( #ident ) }
                }
                Key::StringLiteral(literal) => {
                    quote! { ::surgeist_js::js_string!( #literal ) }
                }
            };
            let (value, setup) = if field.value.needs_context() {
                (
                    field.value.output(Some(&field_context))?,
                    quote! { let mut #field_context = #object.context(); },
                )
            } else {
                (field.value.output(None)?, TokenStream::new())
            };
            body = value.bind(
                &property_value,
                &setup,
                quote! {
                    #object.property( #key, #property_value, ::surgeist_js::property::Attribute::all() );
                    #body
                },
            );
        }

        Ok(quote! {
            {
                let mut #object = ::surgeist_js::object::ObjectInitializer::new( #c_ident.reborrow() );
                #body
            }
        })
    }
}

impl Parse for Object {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let content;
        let brace = braced!(content in input);

        Ok(Self {
            _brace: brace,
            fields: Punctuated::parse_terminated(&content)?,
        })
    }
}

/// An array declaration.
struct Array {
    _bracket: Bracket,
    elems: Punctuated<Value, Token![,]>,
}

impl Array {
    fn output(&self, context: Option<&Ident>) -> syn::Result<TokenStream> {
        let Some(c_ident) = context else {
            return Err(syn::Error::new(
                Span::call_site(),
                "Need to specify a ContextAccess expression.",
            ));
        };

        let items = self
            .elems
            .iter()
            .map(|item| item.output(context))
            .collect::<syn::Result<Vec<_>>>()?;
        if items
            .iter()
            .all(|item| matches!(item, ExpandedValue::Plain(_)))
        {
            let items = items.into_iter().map(ExpandedValue::tokens);
            return Ok(quote! {
                ::surgeist_js::object::builtins::JsArray::from_iter( [ #(#items),* ], #c_ident.reborrow() )
            });
        }

        let items = items.into_iter().map(|item| match item {
            ExpandedValue::Plain(value) => quote! { ::core::result::Result::Ok(#value) },
            ExpandedValue::Checked(value) => value,
        });
        let elements = Ident::new("__surgeist_js_elements", Span::mixed_site());
        // Keep embedded expressions in array element position so their temporaries
        // live through sibling expressions and their control flow stays with the caller.
        Ok(quote! {
            ::core::iter::IntoIterator::into_iter([ #(#items),* ])
                .collect::<::surgeist_js::JsResult<::std::vec::Vec<_>>>()
                .and_then(|#elements| {
                    ::surgeist_js::object::builtins::JsArray::from_iter(#elements, #c_ident.reborrow())
                })
        })
    }
}

impl Parse for Array {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let content;
        let bracket = bracketed!(content in input);
        Ok(Self {
            _bracket: bracket,
            elems: Punctuated::parse_terminated(&content)?,
        })
    }
}

/// The result of parsing the full `js_value!()` macro arguments.
struct JsValue {
    value: Value,
    context: Option<Expr>,
}

impl JsValue {
    fn output(&self) -> TokenStream {
        with_context(
            self.context.as_ref(),
            self.value.needs_context(),
            |context| self.value.output(context).map(ExpandedValue::tokens),
        )
        .unwrap_or_else(|err| err.to_compile_error())
    }
}

impl Parse for JsValue {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let value = input.parse()?;
        let mut context = None;

        if !input.is_empty() {
            input.parse::<Token![,]>()?;
            context = Some(input.parse()?);
        }

        Ok(Self { value, context })
    }
}

/// The result of parsing the `js_object!()` arguments.
struct JsObject {
    value: Object,
    context: Option<Expr>,
}

impl JsObject {
    fn output(&self) -> TokenStream {
        with_context(self.context.as_ref(), true, |context| {
            self.value.output(context)
        })
        .unwrap_or_else(|err| err.to_compile_error())
    }
}

impl Parse for JsObject {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let value = input.parse()?;
        let mut context = None;

        if !input.is_empty() {
            input.parse::<Token![,]>()?;
            context = Some(input.parse()?);
        }

        Ok(Self { value, context })
    }
}

/// Keep temporary owners in the scrutinee alive while generated construction
/// uses their capability. User expressions remain in the caller's control flow.
fn with_context(
    context: Option<&Expr>,
    needs_context: bool,
    output: impl FnOnce(Option<&Ident>) -> syn::Result<TokenStream>,
) -> syn::Result<TokenStream> {
    let Some(context) = context else {
        return output(None);
    };
    let input = Ident::new("__surgeist_js_context_input", Span::mixed_site());
    let access = Ident::new("__surgeist_js_context_access", Span::mixed_site());
    let binding = if needs_context {
        quote! { let mut #access: ::surgeist_js::ContextAccess<'_> = #input; }
    } else {
        quote! { let #access: ::surgeist_js::ContextAccess<'_> = #input; }
    };
    let body = output(Some(&access))?;
    Ok(quote! {
        match (#context) {
            #input => {
                #binding
                #body
            }
        }
    })
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn js_object_impl(input: TokenStream) -> TokenStream {
    parse2::<JsObject>(input).map_or_else(|e| e.to_compile_error(), |v| v.output())
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn js_value_impl(input: TokenStream) -> TokenStream {
    parse2::<JsValue>(input).map_or_else(|e| e.to_compile_error(), |v| v.output())
}
