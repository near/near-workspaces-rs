use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{quote, quote_spanned, ToTokens};

enum Flavor {
    Sandbox,
    Testnet,
    Mainnet,
}

fn parse_knobs(
    mut input: syn::ItemFn,
    args: syn::AttributeArgs,
    is_test: bool,
) -> Result<TokenStream, syn::Error> {
    if input.sig.asyncness.take().is_none() {
        let msg = "the `async` keyword is missing from the function declaration";
        return Err(syn::Error::new_spanned(input.sig.fn_token, msg));
    }

    let mut flavor = None;
    for arg in args {
        match arg {
            syn::NestedMeta::Meta(syn::Meta::NameValue(namevalue)) => {
                let ident = namevalue
                    .path
                    .get_ident()
                    .ok_or_else(|| {
                        syn::Error::new_spanned(&namevalue, "Must have specified ident")
                    })?
                    .to_string();

                let msg = format!("Not expected to received a value for: {}", ident.as_str());
                return Err(syn::Error::new_spanned(namevalue, msg));
            }
            syn::NestedMeta::Meta(syn::Meta::Path(path)) => {
                let name = path
                    .get_ident()
                    .ok_or_else(|| syn::Error::new_spanned(&path, "Must have specified ident"))?
                    .to_string()
                    .to_lowercase();
                match name.as_str() {
                    "sandbox" => {
                        flavor = Some(Flavor::Sandbox);
                    }
                    "testnet" => {
                        flavor = Some(Flavor::Testnet);
                    }
                    "mainnet" => {
                        flavor = Some(Flavor::Mainnet);
                    }
                    name => {
                        let msg = format!("Unknown attribute {} is specified; expected one of: `sandbox`, `testnet`, `mainnet`", name);
                        return Err(syn::Error::new_spanned(path, msg));
                    }
                };
            }
            other => {
                return Err(syn::Error::new_spanned(
                    other,
                    "Unknown attribute inside the macro",
                ));
            }
        }
    }

    // TODO: env flag for configuring which runtime flavor
    if flavor.is_none() {
        flavor = Some(Flavor::Sandbox);
    }

    // If type mismatch occurs, the current rustc points to the last statement.
    let (last_stmt_start_span, last_stmt_end_span) = {
        let mut last_stmt = input
            .block
            .stmts
            .last()
            .map(ToTokens::into_token_stream)
            .unwrap_or_default()
            .into_iter();
        // `Span` on stable Rust has a limitation that only points to the first
        // token, not the whole tokens. We can work around this limitation by
        // using the first/last span of the tokens like
        // `syn::Error::new_spanned` does.
        let start = last_stmt.next().map_or_else(Span::call_site, |t| t.span());
        let end = last_stmt.last().map_or(start, |t| t.span());
        (start, end)
    };

    let rt = match flavor.unwrap() {
        Flavor::Sandbox => quote_spanned! {last_stmt_start_span=>
            let mut rt = runner::SandboxRuntime::default();
            let _ = rt.run().unwrap();
        },
        // TODO: Add further implementations for mainnet and testnet
        _ => unimplemented!(),
    };

    let header = if is_test {
        quote! {
            #[::core::prelude::v1::test]
        }
    } else {
        quote! {}
    };

    let body = &input.block;
    let brace_token = input.block.brace_token;
    input.block = syn::parse2(quote_spanned! {last_stmt_end_span=>
        {
            #rt
            let body = async #body;
            let rt = tokio::runtime::Runtime::new().unwrap();
            let local = tokio::task::LocalSet::new();
            local.block_on(&rt, body);
        }
    })
    .expect("Parsing failure");
    input.block.brace_token = brace_token;

    let result = quote! {
        #header
        #input
    };

    Ok(result.into())
}

pub(crate) fn test(args: TokenStream, item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::ItemFn);
    let args = syn::parse_macro_input!(args as syn::AttributeArgs);

    // Check whether a #[test] is supplied as well
    for attr in &input.attrs {
        if attr.path.is_ident("test") {
            let msg = "second test attribute is supplied";
            return syn::Error::new_spanned(&attr, msg)
                .to_compile_error()
                .into();
        }
    }

    parse_knobs(input, args, true).unwrap_or_else(|e| e.to_compile_error().into())
}

#[cfg(not(test))] // Work around for rust-lang/rust#62127
pub(crate) fn main(args: TokenStream, item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::ItemFn);
    let args = syn::parse_macro_input!(args as syn::AttributeArgs);

    parse_knobs(input, args, false).unwrap_or_else(|e| e.to_compile_error().into())
}
