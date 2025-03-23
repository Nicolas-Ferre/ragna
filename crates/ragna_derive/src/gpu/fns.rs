use crate::gpu::{attrs, GpuModule};
use syn::spanned::Spanned;
use syn::{fold, parse_quote_spanned, Block, Expr, FnArg, ItemFn, Pat, Signature, Token, Type};

pub(crate) fn item_to_gpu(mut item: ItemFn, module: &mut GpuModule) -> ItemFn {
    module.current_fn_signature = Some(item.sig.clone());
    if item.sig.constness.is_some() {
        return item;
    }
    if item.attrs.iter().any(attrs::is_compute) {
        module.compute_fns.push(item.sig.ident.clone());
    }
    let span = item.span();
    item.attrs = item.attrs
        .into_iter()
        .filter(|attr| !attrs::is_compute(attr))
        .chain([parse_quote_spanned! { span => #[allow(unused_braces)] }])
        .collect();
    item = fold::fold_item_fn(module, item);
    signature_impl_to_gpu(&mut item.block, &mut item.sig, module);
    module.current_fn_signature = None;
    item
}

pub(crate) fn signature_impl_to_gpu(
    block: &mut Block,
    sig: &mut Signature,
    module: &mut GpuModule,
) {
    for arg in sig.inputs.iter_mut().rev() {
        if let (false, Some(ident)) = (is_arg_ref(arg), arg_ident(arg, module)) {
            make_arg_mut(arg);
            block.stmts.insert(
                0,
                parse_quote_spanned! { ident.span() => #ident = ::ragna::create_var(#ident); },
            );
        }
    }
}

pub(crate) fn is_arg_ref(arg: &FnArg) -> bool {
    match arg {
        FnArg::Receiver(receiver) => receiver.reference.is_some(),
        FnArg::Typed(ty) => matches!(*ty.ty, Type::Reference(_)),
    }
}

pub(crate) fn arg_ident(arg: &FnArg, module: &mut GpuModule) -> Option<Expr> {
    match arg {
        FnArg::Receiver(receiver) => Some(parse_quote_spanned! { receiver.span() => self }),
        FnArg::Typed(ty) => {
            if let Pat::Ident(ident) = &*ty.pat {
                let ident = &ident.ident;
                Some(parse_quote_spanned! { ident.span() => #ident })
            } else {
                module
                    .errors
                    .push(syn::Error::new(arg.span(), "unsupported parameter"));
                None
            }
        }
    }
}

pub(crate) fn make_arg_mut(arg: &mut FnArg) {
    match arg {
        FnArg::Receiver(receiver) => receiver.mutability = Some(Token![mut](receiver.span())),
        FnArg::Typed(ty) => *ty = parse_quote_spanned! { ty.span() => mut #ty },
    }
}
