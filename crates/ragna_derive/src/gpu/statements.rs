use crate::gpu::GpuModule;
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
        Stmt::Expr(expr, semi) => Stmt::Expr(
            if semi.is_none() {
                returned_expr_to_gpu(expr, module)
            } else {
                module.fold_expr(expr)
            },
            semi,
        ),
        stmt => {
            module
                .errors
                .push(syn::Error::new(stmt.span(), "unsupported item"));
            stmt
        }
    }
}

fn returned_expr_to_gpu(expr: Expr, module: &mut GpuModule) -> Expr {
    if module.is_current_fn_returning_copy() {
        let expr = module.fold_expr(expr);
        parse_quote_spanned! {
            expr.span() => ::ragna::create_var(#expr)
        }
    } else {
        module.fold_expr(expr)
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
