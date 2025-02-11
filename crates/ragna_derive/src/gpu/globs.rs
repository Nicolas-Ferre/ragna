use crate::gpu::{types, GpuModule};
use std::mem;
use syn::fold::Fold;
use syn::spanned::Spanned;
use syn::{parse_quote_spanned, Generics, Item, ItemConst, ItemStatic, LitInt, Token};

pub(crate) fn item_to_gpu(item: ItemStatic, module: &mut GpuModule) -> Item {
    module.globs.push(item.ident.clone());
    let id = LitInt::new(&module.next_id().to_string(), item.span());
    let expr = module.fold_expr(*item.expr);
    let statements = mem::take(&mut module.extracted_statements);
    Item::Const(ItemConst {
        attrs: item.attrs,
        vis: item.vis,
        const_token: Token![const](item.static_token.span),
        ident: item.ident,
        generics: Generics::default(),
        colon_token: item.colon_token,
        ty: types::mut_to_gpu(&item.ty).into(),
        eq_token: item.eq_token,
        expr: parse_quote_spanned! {
            expr.span() => ::ragna::Gpu::glob(
                module_path!(),
                #id,
                |__ctx|{ #(#statements)* ::ragna::Gpu::var(#expr, __ctx) }
            )
        },
        semi_token: item.semi_token,
    })
}
