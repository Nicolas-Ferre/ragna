use crate::gpu::fns::signature_impl_to_gpu;
use crate::gpu::GpuModule;
use syn::spanned::Spanned;
use syn::{fold, parse_quote_spanned, ItemTrait, TraitItem, TraitItemFn};

pub(crate) fn item_to_gpu(mut item: ItemTrait, module: &mut GpuModule) -> ItemTrait {
    item.items = item
        .items
        .into_iter()
        .map(|item| inner_item_to_gpu(item, module))
        .collect();
    item
}

#[allow(clippy::wildcard_enum_match_arm)]
fn inner_item_to_gpu(item: TraitItem, module: &mut GpuModule) -> TraitItem {
    match item {
        TraitItem::Fn(fn_) => fn_to_gpu(fn_, module).into(),
        item @ (TraitItem::Const(_) | TraitItem::Type(_)) => item,
        item => {
            module
                .errors
                .push(syn::Error::new(item.span(), "unsupported item"));
            fold::fold_trait_item(module, item)
        }
    }
}

fn fn_to_gpu(mut item: TraitItemFn, module: &mut GpuModule) -> TraitItemFn {
    module.current_fn_signature = Some(item.sig.clone());
    item.attrs
        .push(parse_quote_spanned! { item.span() => #[allow(unused_braces)] });
    item = fold::fold_trait_item_fn(module, item);
    if let Some(block) = &mut item.default {
        signature_impl_to_gpu(block, &mut item.sig, module);
    }
    module.current_fn_signature = None;
    item
}
