mod entry;

use proc_macro::TokenStream;


#[proc_macro_attribute]
pub fn test(args: TokenStream, item: TokenStream) -> TokenStream {
    entry::test(args, item, true)
}
