use crate::gpu::fns::signature_impl_to_gpu;
use crate::gpu::GpuModule;
use syn::spanned::Spanned;
use syn::{fold, parse_quote_spanned, ImplItem, ImplItemFn, ItemImpl};

pub(crate) fn block_to_gpu(mut block: ItemImpl, module: &mut GpuModule) -> ItemImpl {
    block.items = block
        .items
        .into_iter()
        .map(|item| item_to_gpu(item, module))
        .collect();
    block
}

#[allow(clippy::wildcard_enum_match_arm)]
fn item_to_gpu(item: ImplItem, module: &mut GpuModule) -> ImplItem {
    match item {
        item @ (ImplItem::Const(_) | ImplItem::Type(_)) => item,
        ImplItem::Fn(fn_) => fn_to_gpu(fn_, module).into(),
        item => {
            module
                .errors
                .push(syn::Error::new(item.span(), "unsupported item"));
            fold::fold_impl_item(module, item)
        }
    }
}

fn fn_to_gpu(mut item: ImplItemFn, module: &mut GpuModule) -> ImplItemFn {
    module.current_fn_signature = Some(item.sig.clone());
    if item.sig.constness.is_some() {
        return item;
    }
    let span = item.span();
    item.attrs
        .push(parse_quote_spanned! { span => #[allow(unused_braces)] });
    item = fold::fold_impl_item_fn(module, item);
    signature_impl_to_gpu(&mut item.block, &mut item.sig, module);
    module.current_fn_signature = None;
    item
}
