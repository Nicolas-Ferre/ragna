use crate::gpu::{attrs, generics, types, GpuModule};
use proc_macro2::Ident;
use syn::fold::Fold;
use syn::spanned::Spanned;
use syn::{parse_quote_spanned, Block, FnArg, ItemFn, Pat, ReturnType, Signature};

pub(crate) fn item_to_gpu(mut item: ItemFn, module: &mut GpuModule) -> ItemFn {
    if item.attrs.iter().any(attrs::is_compute) {
        module.compute_fns.push(item.sig.ident.clone());
    } else {
        add_fn_param_vars(&item.sig, &mut item.block);
    }
    let span = item.span();
    item.attrs = item
        .attrs
        .into_iter()
        .filter(|attr| !attrs::is_compute(attr))
        .chain([parse_quote_spanned! { span => #[allow(const_item_mutation, unused_braces)] }])
        .collect();
    item.sig = fn_sig_to_gpu(item.sig, module);
    item.block = module.fold_block(*item.block).into();
    item
}

pub(crate) fn add_fn_param_vars(sig: &Signature, block: &mut Block) {
    for arg in sig.inputs.iter().rev() {
        if arg_ident(arg).map_or(true, |ident| ident != "__ctx") {
            if let Some(ident) = arg_ident(arg) {
                block.stmts.insert(
                    0,
                    parse_quote_spanned! { ident.span() => let mut #ident = #ident; },
                );
            }
        }
    }
}

pub(crate) fn fn_sig_to_gpu(mut sig: Signature, module: &mut GpuModule) -> Signature {
    let span = sig.span();
    sig.inputs = sig
        .inputs
        .into_iter()
        .map(|arg| arg_to_gpu(arg, module))
        .chain([parse_quote_spanned! { span => __ctx: &mut ::ragna::GpuContext }])
        .collect();
    sig.generics = generics::params_to_gpu(sig.generics, module);
    if let ReturnType::Type(_, ty) = &mut sig.output {
        if !types::is_self(ty) {
            *ty = types::mut_to_gpu(ty).into();
        }
    }
    sig
}

fn arg_to_gpu(arg: FnArg, module: &mut GpuModule) -> FnArg {
    match arg {
        FnArg::Receiver(arg) => arg.into(),
        FnArg::Typed(mut arg) => {
            if let Pat::Ident(_) = &*arg.pat {
                if !types::is_self(&arg.ty) {
                    arg.ty = types::any_to_gpu(&arg.ty).into();
                }
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
