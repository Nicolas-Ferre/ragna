use crate::gpu::{attrs, GpuModule};
use proc_macro2::Ident;
use syn::fold::Fold;
use syn::spanned::Spanned;
use syn::{parse_quote_spanned, FnArg, ItemFn, Pat, Signature};

pub(crate) fn item_to_gpu(mut item: ItemFn, module: &mut GpuModule) -> ItemFn {
    if item.sig.constness.is_some() {
        return item;
    }
    if item.attrs.iter().any(attrs::is_compute) {
        module.compute_fns.push(item.sig.ident.clone());
    }
    let span = item.span();
    item.attrs = item
        .attrs
        .into_iter()
        .filter(|attr| !attrs::is_compute(attr))
        .chain([parse_quote_spanned! { span => #[allow(unused_braces)] }])
        .collect();
    item.sig = signature_to_gpu(item.sig, module);
    for arg in item.sig.inputs.iter().rev() {
        if let Some(ident) = arg_ident(arg, module) {
            item.block.stmts.insert(
                0,
                parse_quote_spanned! { ident.span() => let #ident = #ident; },
            );
        }
    }
    item.block = module.fold_block(*item.block).into();
    item
}

pub(crate) fn signature_to_gpu(mut sig: Signature, module: &mut GpuModule) -> Signature {
    sig.inputs = sig
        .inputs
        .into_iter()
        .map(|arg| arg_to_gpu(arg, module))
        .collect();
    sig
}

fn arg_to_gpu(arg: FnArg, module: &mut GpuModule) -> FnArg {
    match arg {
        FnArg::Receiver(arg) => {
            // to support it, need to extract `self` in `__self` variable and rename all occurrences
            module
                .errors
                .push(syn::Error::new(arg.span(), "unsupported parameter"));
            arg.into()
        }
        FnArg::Typed(arg) => arg.into(),
    }
}

pub(crate) fn arg_ident<'a>(arg: &'a FnArg, module: &mut GpuModule) -> Option<&'a Ident> {
    if let FnArg::Typed(pat) = arg {
        if let Pat::Ident(ident) = &*pat.pat {
            Some(&ident.ident)
        } else {
            module
                .errors
                .push(syn::Error::new(arg.span(), "unsupported parameter"));
            None
        }
    } else {
        None
    }
}
