use crate::gpu::{attrs, types, GpuModule};
use proc_macro2::Ident;
use syn::fold::Fold;
use syn::spanned::Spanned;
use syn::{parse_quote_spanned, FnArg, ItemFn, Pat, ReturnType, Signature};

pub(crate) fn item_to_gpu(mut item: ItemFn, module: &mut GpuModule) -> ItemFn {
    if item.attrs.iter().any(attrs::is_compute) {
        module.compute_fns.push(item.sig.ident.clone());
    } else {
        for arg in item.sig.inputs.iter().rev() {
            if let Some(ident) = arg_ident(arg) {
                item.block.stmts.insert(
                    0,
                    parse_quote_spanned! { ident.span() => let mut #ident = #ident; },
                );
            }
        }
    }
    let span = item.span();
    item.attrs = item
        .attrs
        .into_iter()
        .filter(|attr| !attrs::is_compute(attr))
        .chain([parse_quote_spanned! { span => #[allow(const_item_mutation, unused_braces)] }])
        .collect();
    item.sig = signature_to_gpu(item.sig, module);
    item.block = module.fold_block(*item.block).into();
    item
}

pub(crate) fn signature_to_gpu(mut sig: Signature, module: &mut GpuModule) -> Signature {
    let span = sig.span();
    sig.inputs = sig
        .inputs
        .into_iter()
        .map(|arg| arg_to_gpu(arg, module))
        .chain([parse_quote_spanned! { span => __ctx: &mut ::ragna::GpuContext }])
        .collect();
    if let ReturnType::Type(_, ty) = &mut sig.output {
        *ty = types::mut_to_gpu(ty).into();
    }
    sig
}

fn arg_to_gpu(arg: FnArg, module: &mut GpuModule) -> FnArg {
    match arg {
        FnArg::Receiver(arg) => {
            module
                .errors
                .push(syn::Error::new(arg.span(), "unsupported parameter"));
            arg.into()
        }
        FnArg::Typed(mut arg) => {
            if let Pat::Ident(_) = &*arg.pat {
                arg.ty = types::any_to_gpu(&arg.ty).into();
            } else {
                module
                    .errors
                    .push(syn::Error::new(arg.pat.span(), "unsupported pattern"));
            }
            arg.into()
        }
    }
}

pub(crate) fn arg_ident(arg: &FnArg) -> Option<&Ident> {
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
