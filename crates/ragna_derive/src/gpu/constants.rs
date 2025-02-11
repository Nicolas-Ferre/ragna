use crate::gpu::types;
use quote::ToTokens;
use syn::spanned::Spanned;
use syn::{parse_quote_spanned, Expr, ItemConst};

pub(crate) fn value_to_gpu(value: impl ToTokens) -> Expr {
    parse_quote_spanned! { value.span() => ::ragna::Gpu::constant(#value) }
}

pub(crate) fn item_to_gpu(mut item: ItemConst) -> ItemConst {
    item.ty = types::const_to_gpu(&item.ty).into();
    item.expr = value_to_gpu(item.expr).into();
    item
}
