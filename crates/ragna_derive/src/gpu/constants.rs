use quote::ToTokens;
use syn::spanned::Spanned;
use syn::{parse_quote_spanned, Expr, ItemConst};

pub(crate) fn item_to_gpu(mut item: ItemConst) -> ItemConst {
    let ty = &item.ty;
    let expr = &item.expr;
    item.expr = parse_quote_spanned! { expr.span() => <#ty>::from_cpu(#expr) };
    item
}

pub(crate) fn expr_to_gpu(value: impl ToTokens) -> Expr {
    parse_quote_spanned! { value.span() => ::ragna::Cpu::to_gpu(#value) }
}
