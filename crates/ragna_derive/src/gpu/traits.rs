use crate::gpu::fns::{add_fn_param_vars, fn_sig_to_gpu};
use crate::gpu::statements::block_to_gpu;
use crate::gpu::{generics, types, GpuModule};
use proc_macro2::{Ident, Span};
use quote::format_ident;
use std::mem;
use syn::spanned::Spanned;
use syn::{parse_quote_spanned, ItemTrait, TraitItem, TraitItemConst, TraitItemFn, TraitItemType};

pub(crate) fn item_to_gpu(item: ItemTrait, module: &mut GpuModule) -> ItemTrait {
    let mut gpu_item = item.clone();
    let span = gpu_item.span();
    // gpu_item.ident = format_ident!(
    //     "{}{}",
    //     Ident::new("Gpu", gpu_item.ident.span()),
    //     gpu_item.ident
    // );
    gpu_item.generics = generics::params_to_gpu(gpu_item.generics, module);
    gpu_item.items = mem::take(&mut gpu_item.items)
        .into_iter()
        .map(|item| inner_item_to_gpu(item, module))
        .chain([cpu_associated_type(span).into()])
        .collect();
    // module.generated_items.push(gpu_item.into());
    gpu_item
}

fn inner_item_to_gpu(item: TraitItem, module: &mut GpuModule) -> TraitItem {
    match item {
        TraitItem::Const(item) => const_to_gpu(item).into(),
        TraitItem::Fn(item) => fn_to_gpu(item, module).into(),
        TraitItem::Type(item) => type_to_gpu(item).into(),
        item @ (TraitItem::Macro(_) | TraitItem::Verbatim(_) | _) => {
            module
                .errors
                .push(syn::Error::new(item.span(), "unsupported item"));
            item
        }
    }
}

fn const_to_gpu(mut item: TraitItemConst) -> TraitItemConst {
    item.ty = types::const_to_gpu(&item.ty);
    item
}

fn fn_to_gpu(mut item: TraitItemFn, module: &mut GpuModule) -> TraitItemFn {
    item.sig = fn_sig_to_gpu(item.sig, module);
    if let Some(mut block) = item.default.take() {
        add_fn_param_vars(&item.sig, &mut block);
        item.default = Some(block_to_gpu(block, module));
    }
    item
}

fn type_to_gpu(mut item: TraitItemType) -> TraitItemType {
    item.bounds
        .push(parse_quote_spanned! { item.span() => ::ragna::GpuType });
    item
}

fn cpu_associated_type(span: Span) -> TraitItemType {
    parse_quote_spanned! { span => type __Cpu: ::ragna::GpuType; }
}
