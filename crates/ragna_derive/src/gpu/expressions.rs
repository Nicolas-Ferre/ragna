use crate::gpu::{vars, GpuModule};
use proc_macro2::{Ident, TokenStream};
use quote::ToTokens;
use std::mem;
use syn::fold::Fold;
use syn::spanned::Spanned;
use syn::{
    fold, parse_quote_spanned, BinOp, Expr, ExprAssign, ExprBinary, ExprBreak, ExprIf, ExprUnary,
    ExprWhile, Stmt,
};

macro_rules! transform_binary_expr {
    ($module:ident, $expr:expr, $left:ident, $right:ident, $($new_expr:tt)+) => {{
        let attrs = &$expr.attrs;
        let $left = &$expr.left;
        let $right = &$expr.right;
        $module.fold_expr(parse_quote_spanned! {
            $expr.span() => #(#attrs)* $($new_expr)+
        })
    }};
}

#[allow(clippy::wildcard_enum_match_arm)]
pub(crate) fn expr_to_gpu(expr: Expr, module: &mut GpuModule) -> Expr {
    match expr {
        Expr::Lit(expr) => literal_to_gpu(expr),
        Expr::Assign(expr) => assign_to_gpu(expr, module),
        Expr::Unary(expr) => unary_to_gpu(expr, module),
        Expr::Binary(expr) => binary_to_gpu(expr, module),
        Expr::If(expr) => Expr::Verbatim(if_to_gpu(expr, module)),
        Expr::While(expr) => Expr::Verbatim(while_to_gpu(expr, module)),
        Expr::Break(expr) => Expr::Verbatim(break_to_gpu(expr, module)),
        expr @ (Expr::Path(_)
        | Expr::Paren(_)
        | Expr::Call(_)
        | Expr::MethodCall(_)
        | Expr::Reference(_)
        | Expr::Block(_)
        | Expr::Verbatim(_)) => fold::fold_expr(module, expr),
        expr => {
            module
                .errors
                .push(syn::Error::new(expr.span(), "unsupported expression"));
            fold::fold_expr(module, expr)
        }
    }
}

fn literal_to_gpu(value: impl ToTokens) -> Expr {
    parse_quote_spanned! { value.span() => ::ragna::Cpu::to_gpu(#value) }
}

fn assign_to_gpu(expr: ExprAssign, module: &mut GpuModule) -> Expr {
    let span = expr.span();
    let attrs = &expr.attrs;
    let left = &expr.left;
    let right = module.fold_expr(*expr.right);
    parse_quote_spanned! { span => #(#attrs)* (::ragna::Gpu::assign(#left, #right)) }
}

fn unary_to_gpu(expr: ExprUnary, module: &mut GpuModule) -> Expr {
    if matches!(*expr.expr, Expr::Lit(_)) {
        // to avoid out of range error with -2_147_483_648_i32 value
        literal_to_gpu(expr)
    } else {
        fold::fold_expr_unary(module, expr).into()
    }
}

fn binary_to_gpu(expr: ExprBinary, module: &mut GpuModule) -> Expr {
    let span = expr.span();
    match &expr.op {
        BinOp::And(_) => {
            let attrs = expr.attrs;
            let left = module.fold_expr(*expr.left);
            let right = module.fold_expr(*expr.right);
            parse_quote_spanned! { span => #(#attrs)* ::ragna::Bool::and(#left, #right) }
        }
        BinOp::Or(_) => {
            let attrs = expr.attrs;
            let left = module.fold_expr(*expr.left);
            let right = module.fold_expr(*expr.right);
            parse_quote_spanned! { span => #(#attrs)* ::ragna::Bool::or(#left, #right) }
        }
        BinOp::Eq(_) => transform_bool_binary_op(expr, "Equal", module),
        BinOp::Gt(_) => transform_bool_binary_op(expr, "GreaterThan", module),
        BinOp::Ne(_) => transform_binary_expr!(module, expr, l, r, (!(#l == #r))),
        BinOp::Lt(_) => transform_binary_expr!(module, expr, l, r, (!(#l > #r || #l == #r))),
        BinOp::Le(_) => transform_binary_expr!(module, expr, l, r, (!(#l > #r))),
        BinOp::Ge(_) => transform_binary_expr!(module, expr, l, r, (#l > #r || #l == #r)),
        BinOp::AddAssign(_) => transform_binary_expr!(module, expr, l, r, (#l = #l + #r)),
        BinOp::SubAssign(_) => transform_binary_expr!(module, expr, l, r, (#l = #l - #r)),
        BinOp::MulAssign(_) => transform_binary_expr!(module, expr, l, r, (#l = #l * #r)),
        BinOp::DivAssign(_) => transform_binary_expr!(module, expr, l, r, (#l = #l / #r)),
        BinOp::RemAssign(_) => transform_binary_expr!(module, expr, l, r, (#l = #l % #r)),
        BinOp::Add(_) | BinOp::Sub(_) | BinOp::Mul(_) | BinOp::Div(_) | BinOp::Rem(_) => {
            fold::fold_expr_binary(module, expr).into()
        }
        BinOp::BitXor(_)
        | BinOp::BitAnd(_)
        | BinOp::BitOr(_)
        | BinOp::Shl(_)
        | BinOp::Shr(_)
        | BinOp::BitXorAssign(_)
        | BinOp::BitAndAssign(_)
        | BinOp::BitOrAssign(_)
        | BinOp::ShlAssign(_)
        | BinOp::ShrAssign(_)
        | _ => {
            module.errors.push(syn::Error::new(
                expr.op.span(),
                "unsupported binary operator",
            ));
            fold::fold_expr_binary(module, expr).into()
        }
    }
}

fn transform_bool_binary_op(expr: ExprBinary, trait_: &str, module: &mut GpuModule) -> Expr {
    let span = expr.span();
    let trait_ident = Ident::new(trait_, span);
    let attrs = expr.attrs;
    let left_expr = module.fold_expr(*expr.left);
    let right_expr = module.fold_expr(*expr.right);
    let var_ident = vars::generate_ident(span, module);
    module.extracted_statements.push(parse_quote_spanned! {
        span =>
        let #var_ident = #(#attrs)* ::ragna::#trait_ident::apply(#left_expr, #right_expr);
    });
    parse_quote_spanned! { span => #var_ident }
}

fn if_to_gpu(expr: ExprIf, module: &mut GpuModule) -> TokenStream {
    let span = expr.span();
    let attrs = expr.attrs;
    let var_ident = vars::generate_ident(span, module);
    let is_returning_value = !expr.then_branch.stmts.is_empty()
        && matches!(expr.then_branch.stmts[0], Stmt::Expr(_, None));
    let cond = module.fold_expr(*expr.cond);
    let cond_statements = mem::take(&mut module.extracted_statements);
    let then_branch = expr.then_branch;
    let new_then_branch = module.fold_stmt(if is_returning_value {
        parse_quote_spanned! { then_branch.span() => #var_ident = #then_branch; }
    } else {
        parse_quote_spanned! { then_branch.span() => #then_branch; }
    });
    let else_branch: Option<TokenStream> = if let Some((else_kw, else_expr)) = expr.else_branch {
        let new_else_expr = module.fold_stmt(if is_returning_value {
            parse_quote_spanned! { then_branch.span() => #var_ident = #else_expr; }
        } else {
            parse_quote_spanned! { then_branch.span() => #else_expr; }
        });
        Some(parse_quote_spanned! { else_kw.span() => ::ragna::else_block(); #new_else_expr })
    } else {
        None
    };
    if is_returning_value {
        parse_quote_spanned! {
            span =>
            #(#attrs)*
            {
                let #var_ident = ::ragna::Gpu::create_uninit_var();
                #(#cond_statements)*
                ::ragna::if_block(#cond);
                #new_then_branch
                #else_branch
                ::ragna::end_block();
                #var_ident
            }
        }
    } else {
        parse_quote_spanned! {
            span =>
            #(#attrs)*
            #[allow(redundant_semicolons)]
            {
                #(#cond_statements)*
                ::ragna::if_block(#cond);
                #new_then_branch
                #else_branch
                ::ragna::end_block();
            };
        }
    }
}

fn while_to_gpu(expr: ExprWhile, module: &mut GpuModule) -> TokenStream {
    let span = expr.span();
    let attrs = expr.attrs;
    let cond = module.fold_expr(*expr.cond);
    let cond_statements = mem::take(&mut module.extracted_statements);
    let body = module.fold_block(expr.body);
    if let Some(label) = expr.label {
        module
            .errors
            .push(syn::Error::new(label.span(), "labels not supported"));
    }
    parse_quote_spanned! {
        span =>
        #(#attrs)*
        {
            ::ragna::loop_block();
            {
                #[allow(clippy::no_effect_underscore_binding)]
                let __loop = (); // to ensure `break` and `continue` are called from inside the loop
                #(#cond_statements)*
                ::ragna::if_block(!#cond);
                ::ragna::break_();
                ::ragna::end_block();
                #body
            };
            ::ragna::end_block();
        };
    }
}

fn break_to_gpu(expr: ExprBreak, module: &mut GpuModule) -> TokenStream {
    let span = expr.span();
    let attrs = expr.attrs;
    if let Some(label) = expr.label {
        module
            .errors
            .push(syn::Error::new(label.span(), "labels not supported"));
    }
    if let Some(expr) = expr.expr {
        module.errors.push(syn::Error::new(
            expr.span(),
            "break expressions not supported",
        ));
    }
    parse_quote_spanned! {
        span =>
        #(#attrs)*
        {
            #[allow(path_statements)] __loop; // to ensure `break` is called from inside a loop
            ::ragna::break_();
        }
    }
}
