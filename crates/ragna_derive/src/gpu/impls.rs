use crate::gpu::{generics, GpuModule};
use syn::spanned::Spanned;
use syn::{parse_quote_spanned, ItemImpl};

pub(crate) fn item_to_gpu(mut item: ItemImpl, module: &mut GpuModule) -> ItemImpl {
    item.generics = generics::params_to_gpu(item.generics, module);
    item.generics
        .params
        .push(parse_quote_spanned! {item.span() => __M: 'static });
    item
}
