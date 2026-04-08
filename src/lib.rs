//! # essay
//!
//! Generates tests for functions from attribute macros.
//!
//! The crate is runtime-agnostic and knows nothing about databases, HTTP clients or async runtimes.
//!
//! I built this crate to get [sqlx](https://crates.io/crates/sqlx)-like checked queries for other database clients.
//!
//! ## Usage
//!
//! ```rust,ignore
//! #[essay::essay(cases(
//!     "insert_cpu" => (duckdb_connection(), "cpu", 78.5),
//!     "insert_memory" => (duckdb_connection(), "memory", 45.2) -> check_metric_name_memory,
//! ))]
//! fn insert_metric(conn: Connection, name: &str, val: f64) -> Result<Metric> {
//!     // ...
//! }
//! ```
//!
//! The generated test cases can be run via `cargo test`. See [examples](https://github.com/myFavShrimp/essay/tree/main/examples) for complete use cases.
//!
//! ## Async
//!
//! Async is auto-detected from the function signature. `test_attr` can be used to set the test runtime.
//!     
//! ```rust,ignore
//! #[essay::essay(
//!     test_attr = tokio::test,
//!     cases(
//!         "create_user" => (get_pool().await, "alice"),
//!     ),
//! )]
//! async fn create_user(pool: PgPool, username: &str) -> Result<User> {
//!     // ...
//! }
//! ```
//!
//! ## Attribute syntax
//!
//! ```rust,ignore
//! #[essay::essay(
//!     test_attr = tokio::test,           // Custom test attribute (default is `test`)
//!     cases(
//!         "name" => (arg1) -> assert_fn, // custom assert function
//!         "name2" => (arg1, arg2),       // default is_ok() assertion
//!     )
//! )]
//! ```

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{
    Expr, ItemFn, LitStr, Token,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
};

struct TestCase {
    name: String,
    arguments: Vec<Expr>,
    assert_function: Option<syn::Path>,
}

struct AutoTestAttribute {
    test_attribute: Option<syn::Path>,
    test_cases: Vec<TestCase>,
}

fn parse_case(name: String, input: ParseStream) -> syn::Result<TestCase> {
    let mut arguments = Vec::new();
    let mut assert_function = None;

    if !input.is_empty() && input.peek(syn::token::Paren) {
        let content;
        syn::parenthesized!(content in input);

        let expressions = Punctuated::<Expr, Token![,]>::parse_terminated(&content)?;

        arguments = expressions.into_iter().collect();
    }

    if input.peek(Token![->]) {
        input.parse::<Token![->]>()?;

        assert_function = Some(input.parse::<syn::Path>()?);
    }

    Ok(TestCase {
        name,
        arguments,
        assert_function,
    })
}

impl Parse for AutoTestAttribute {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut test_attribute = None;
        let mut test_cases = Vec::new();

        while !input.is_empty() {
            let key: syn::Ident = input.parse()?;

            match key.to_string().as_str() {
                "test_attr" => {
                    input.parse::<Token![=]>()?;

                    test_attribute = Some(input.parse::<syn::Path>()?);
                }
                "cases" => {
                    let content;
                    syn::parenthesized!(content in input);

                    while !content.is_empty() {
                        let case_name: LitStr = content.parse()?;

                        content.parse::<Token![=>]>()?;

                        test_cases.push(parse_case(case_name.value(), &content)?);

                        if !content.is_empty() {
                            content.parse::<Token![,]>()?;
                        }
                    }
                }
                other => {
                    return Err(syn::Error::new(
                        key.span(),
                        format!("unknown attribute `{}`", other),
                    ));
                }
            }

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        if test_cases.is_empty() {
            return Err(input.error("missing `cases(...)`, at least one case is required"));
        }

        Ok(AutoTestAttribute {
            test_attribute,
            test_cases,
        })
    }
}

fn build_assertion(func_name: &syn::Ident, assert_fn: &Option<syn::Path>) -> TokenStream2 {
    if let Some(path) = assert_fn {
        quote! { #path(__auto_test_result); }
    } else {
        quote! {
            assert!(
                __auto_test_result.is_ok(),
                "{} failed: {:?}",
                stringify!(#func_name),
                __auto_test_result.unwrap_err(),
            );
        }
    }
}

fn generate_output_tokens(
    attribute: TokenStream2,
    item: TokenStream2,
) -> syn::Result<TokenStream2> {
    let attribute: AutoTestAttribute = syn::parse2(attribute)?;
    let function: ItemFn = syn::parse2(item.clone())?;

    let function_name = &function.sig.ident;
    let is_function_async = function.sig.asyncness.is_some();
    let is_test_async = attribute.test_attribute.is_some() || is_function_async;

    let test_attribute_tokens = match &attribute.test_attribute {
        Some(path) => quote! { #[#path] },
        None => quote! { #[test] },
    };

    let async_keyword = if is_test_async {
        quote!(async)
    } else {
        quote!()
    };

    let func_await = if is_function_async {
        quote!(.await)
    } else {
        quote!()
    };

    let tests: Vec<TokenStream2> = attribute
        .test_cases
        .iter()
        .map(|case| {
            let test_identifier = format_ident!("essay__{}__{}", function_name, case.name);
            let arguments = &case.arguments;
            let invocation = quote! { #function_name(#(#arguments),*) };
            let assertion = build_assertion(function_name, &case.assert_function);

            quote! {
                #[cfg(test)]
                #test_attribute_tokens
                #[allow(non_snake_case)]
                #async_keyword fn #test_identifier() {
                    let __auto_test_result = #invocation #func_await;
                    #assertion
                }
            }
        })
        .collect();

    Ok(quote! {
        #function
        #(#tests)*
    })
}

/// Generates tests for a function by calling it with provided arguments and
/// asserting the result.
///
/// Async is auto-detected from the function signature.
///
/// # Attribute syntax
///
/// ```rust
/// #[essay::essay(
///     test_attr = tokio::test,           // Custom test attribute (default is `test`)
///     cases(
///         "name" => (arg1) -> assert_fn, // custom assert function
///         "name2" => (arg1, arg2),       // default is_ok() assertion
///     )
/// )]
/// ```
///
/// # Example
///
/// ```ignore
/// #[essay::essay(cases(
///     "insert_cpu" => (duckdb_conn(), "cpu", 78.5),
///     "insert_mem" => (duckdb_conn(), "mem", 45.2) -> check_metric,
///     "insert_tmp" => (duckdb_conn(), "tmp", 0.0) -> check_metric,
/// ))]
/// fn insert_metric(conn: Connection, name: &str, val: f64) -> Result<Metric> { ... }
/// ```
#[proc_macro_attribute]
pub fn essay(attr: TokenStream, item: TokenStream) -> TokenStream {
    match generate_output_tokens(attr.into(), item.into()) {
        Ok(ts) => ts.into(),
        Err(e) => e.into_compile_error().into(),
    }
}
