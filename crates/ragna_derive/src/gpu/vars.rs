use crate::gpu::GpuModule;
use proc_macro2::{Ident, Span};

pub(crate) fn generate_ident(span: Span, module: &mut GpuModule) -> Ident {
    Ident::new(&format!("tmp{}", module.next_id()), span)
}
