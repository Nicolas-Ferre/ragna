use crate::gpu::{fns, GpuModule};
use quote::quote;
use syn::spanned::Spanned;
use syn::{
    parse_quote_spanned, ForeignItem, ForeignItemFn, Item, ItemFn, ItemForeignMod, LitStr,
    ReturnType,
};

pub(crate) fn mod_to_gpu(item: ItemForeignMod, module: &mut GpuModule) -> Item {
    check_abi(&item, module);
    for item in item.items {
        if let ForeignItem::Fn(item) = item {
            fn_to_gpu(item, module);
        } else {
            module
                .errors
                .push(syn::Error::new(item.span(), "unsupported item"));
        }
    }
    Item::Verbatim(quote! {})
}

fn check_abi(item: &ItemForeignMod, module: &mut GpuModule) {
    if let Some(abi_name) = &item.abi.name {
        if abi_name.value() != "wgsl" {
            module.errors.push(syn::Error::new(
                abi_name.span(),
                "only \"wgsl\" ABI is supported",
            ));
        }
    }
}

fn fn_to_gpu(mut item: ForeignItemFn, module: &mut GpuModule) {
    let param_idents = item
        .sig
        .inputs
        .iter()
        .filter_map(|arg| fns::arg_ident(arg, module).cloned())
        .collect::<Vec<_>>();
    let span = item.span();
    let fn_name = LitStr::new(&item.sig.ident.to_string(), item.sig.ident.span());
    if item.sig.output == ReturnType::Default {
        module.errors.push(syn::Error::new(
            item.sig.ident.span(),
            "function must have a return type",
        ));
        return;
    }
    item.sig = fns::signature_to_gpu(item.sig, module);
    module.generated_items.push(Item::Fn(ItemFn {
        attrs: item.attrs,
        vis: item.vis,
        sig: item.sig,
        block: parse_quote_spanned! { span => {
            ::ragna::call_fn(#fn_name, vec![#(::ragna::Gpu::value(#param_idents)),*], true)
        }},
    }));
}
