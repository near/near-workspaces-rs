
use proc_macro::TokenStream;
use quote::{quote};
use syn::{parse_macro_input, ItemFn};


#[proc_macro_attribute]
pub fn sandbox(_attr: TokenStream, stream: TokenStream) -> TokenStream {
    let input = parse_macro_input!(stream as ItemFn);
    let ItemFn { attrs, vis, sig, block } = input;
    let stmts = &block.stmts;
    (quote! {
        #(#attrs)* #vis #sig {
            let child = sandbox_runner::sandbox_setup();
            #(#stmts)*
        }
    }).into()
}