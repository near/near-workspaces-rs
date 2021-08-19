mod entry;

use proc_macro::TokenStream;


#[proc_macro_attribute]
pub fn test(args: TokenStream, item: TokenStream) -> TokenStream {
    entry::test(args, item, false)
}

#[cfg(not(test))] // Work around for rust-lang/rust#62127
#[proc_macro_attribute]
pub fn main(args: TokenStream, item: TokenStream) -> TokenStream {
    entry::main(args, item, false)
}
