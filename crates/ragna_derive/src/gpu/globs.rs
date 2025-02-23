use crate::gpu::GpuModule;
use std::mem;
use syn::fold::Fold;
use syn::spanned::Spanned;
use syn::{parse_quote_spanned, ItemStatic, LitInt};

pub(crate) fn item_to_gpu(mut item: ItemStatic, module: &mut GpuModule) -> ItemStatic {
    module.globs.push(item.ident.clone());
    let id = LitInt::new(&module.next_id().to_string(), item.span());
    let ty = &item.ty;
    let expr = module.fold_expr(*item.expr);
    let statements = mem::take(&mut module.extracted_statements);
    item.ty = parse_quote_spanned! { ty.span() => ::ragna::Glob<#ty> };
    item.expr = parse_quote_spanned! {
        expr.span() => ::ragna::Glob::new(|| ::ragna::create_glob(
            module_path!(),
            #id,
            ||{ #(#statements)* #expr }
        ))
    };
    item
}
