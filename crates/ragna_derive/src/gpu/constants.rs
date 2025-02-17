use quote::ToTokens;
use syn::spanned::Spanned;
use syn::{parse_quote_spanned, Expr};

pub(crate) fn expr_to_gpu(value: impl ToTokens) -> Expr {
    parse_quote_spanned! { value.span() => ::ragna::Cpu::to_gpu(#value) }
}
