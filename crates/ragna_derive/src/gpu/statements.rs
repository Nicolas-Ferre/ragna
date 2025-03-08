use crate::gpu::GpuModule;
use proc_macro2::TokenTree;
use quote::ToTokens;
use std::mem;
use syn::fold::Fold;
use syn::spanned::Spanned;
use syn::{parse_quote_spanned, Block, Expr, Local, LocalInit, Pat, Stmt};

pub(crate) fn block_to_gpu(mut block: Block, module: &mut GpuModule) -> Block {
    block.stmts = block
        .stmts
        .into_iter()
        .flat_map(|stmt| {
            let stmt = statement_to_gpu(stmt, module); // do not inline this variable
            mem::take(&mut module.extracted_statements)
                .into_iter()
                .chain([stmt])
        })
        .collect();
    block
}

#[allow(clippy::wildcard_enum_match_arm)]
fn statement_to_gpu(stmt: Stmt, module: &mut GpuModule) -> Stmt {
    match stmt {
        Stmt::Local(local) => Stmt::Local(local_to_gpu(local, module)),
        Stmt::Expr(expr, semi) => {
            let expr = module.fold_expr(expr);
            let expr_trees = expr.to_token_stream().into_iter().collect::<Vec<_>>();
            let has_semi = if let TokenTree::Punct(punc) = &expr_trees[expr_trees.len() - 1] {
                punc.as_char() == ';'
            } else {
                false
            };
            Stmt::Expr(
                if !has_semi && semi.is_none() && module.is_current_fn_returning_copy() {
                    parse_quote_spanned! { expr.span() => ::ragna::create_var(#expr) }
                } else {
                    expr
                },
                semi,
            )
        }
        stmt => {
            module
                .errors
                .push(syn::Error::new(stmt.span(), "unsupported item"));
            stmt
        }
    }
}

fn local_to_gpu(local: Local, module: &mut GpuModule) -> Local {
    Local {
        attrs: local.attrs,
        let_token: local.let_token,
        pat: local_pat_to_gpu(local.pat, module),
        init: local.init.map(|init| local_init_to_gpu(init, module)),
        semi_token: local.semi_token,
    }
}

#[allow(clippy::wildcard_enum_match_arm)]
fn local_pat_to_gpu(pat: Pat, module: &mut GpuModule) -> Pat {
    match pat {
        pat @ (Pat::Type(_) | Pat::Ident(_)) => pat,
        pat => {
            module.errors.push(syn::Error::new(
                pat.span(),
                "unsupported variable definition syntax",
            ));
            pat
        }
    }
}

fn local_init_to_gpu(mut init: LocalInit, module: &mut GpuModule) -> LocalInit {
    let expr = module.fold_expr(*init.expr);
    init.expr = if matches!(expr, Expr::Reference(_)) {
        expr.into()
    } else {
        parse_quote_spanned! { expr.span() => ::ragna::create_var(#expr) }
    };
    init
}
