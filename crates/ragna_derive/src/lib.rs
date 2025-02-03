//! Ragna proc macros.

use proc_macro::TokenStream;
use syn::{parse_macro_input, ItemMod};

mod gpu;

#[allow(missing_docs)] // doc available in `ragna` crate
#[proc_macro_attribute]
pub fn gpu(_args: TokenStream, item: TokenStream) -> TokenStream {
    let module = parse_macro_input!(item as ItemMod);
    gpu::gpu(&module).into()
}
