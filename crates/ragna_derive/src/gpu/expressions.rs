use crate::gpu::{constants, vars, GpuModule};
use proc_macro2::Ident;
use syn::fold::Fold;
use syn::spanned::Spanned;
use syn::{fold, parse_quote_spanned, BinOp, Expr, ExprAssign, ExprBinary, ExprUnary, UnOp};

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

pub(crate) fn assign_to_gpu(expr: ExprAssign, module: &mut GpuModule) -> Expr {
    let span = expr.span();
    let attrs = &expr.attrs;
    let left = &expr.left;
    let right = module.fold_expr(*expr.right);
    parse_quote_spanned! { span => #(#attrs)* (::ragna::Gpu::assign(#left, #right)) }
}

pub(crate) fn unary_to_gpu(expr: ExprUnary, module: &mut GpuModule) -> Expr {
    if matches!(*expr.expr, Expr::Lit(_)) {
        // to avoid out of range error with -2_147_483_648_i32 value
        constants::expr_to_gpu(expr)
    } else {
        match &expr.op {
            UnOp::Not(_) | UnOp::Neg(_) => fold::fold_expr_unary(module, expr).into(),
            UnOp::Deref(_) | _ => {
                module.errors.push(syn::Error::new(
                    expr.op.span(),
                    "unsupported unary operator",
                ));
                fold::fold_expr_unary(module, expr).into()
            }
        }
    }
}

pub(crate) fn binary_to_gpu(expr: ExprBinary, module: &mut GpuModule) -> Expr {
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
