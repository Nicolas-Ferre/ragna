use crate::gpu::{fns, GpuModule};
use proc_macro2::Ident;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{
    parse_quote_spanned, FnArg, ForeignItem, ForeignItemFn, Item, ItemFn, ItemForeignMod, LitStr,
    Pat, ReturnType,
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

fn fn_to_gpu(mut item: ForeignItemFn, module: &mut GpuModule) -> ForeignItemFn {
    let param_idents = item
        .sig
        .inputs
        .iter()
        .filter_map(|arg| arg_ident(arg).cloned())
        .collect::<Vec<_>>();
    let span = item.span();
    let fn_name = LitStr::new(&item.sig.ident.to_string(), item.sig.ident.span());
    let semi = match item.sig.output {
        ReturnType::Default => quote_spanned! { span => ; },
        ReturnType::Type(_, _) => quote_spanned! { span => },
    };
    item.sig = fns::signature_to_gpu(item.sig, module);
    module.generated_items.push(Item::Fn(ItemFn {
        attrs: item.attrs.clone(),
        vis: item.vis.clone(),
        sig: item.sig.clone(),
        block: parse_quote_spanned! { span => {
            __ctx.call_fn(#fn_name, vec![#(#param_idents.value()),*]) #semi
        }},
    }));
    item
}

fn arg_ident(arg: &FnArg) -> Option<&Ident> {
    if let FnArg::Typed(pat) = arg {
        if let Pat::Ident(ident) = &*pat.pat {
            Some(&ident.ident)
        } else {
            None
        }
    } else {
        None
    }
}
