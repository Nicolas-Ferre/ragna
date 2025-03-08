use crate::gpu::{vars, GpuModule};
use proc_macro2::{Ident, TokenStream};
use quote::ToTokens;
use std::mem;
use syn::fold::Fold;
use syn::spanned::Spanned;
use syn::{
    fold, parse_quote_spanned, BinOp, Expr, ExprArray, ExprAssign, ExprBinary, ExprBreak, ExprCall,
    ExprContinue, ExprForLoop, ExprIf, ExprLit, ExprRange, ExprUnary, ExprWhile, Lit, LitInt, Pat,
    RangeLimits, Stmt,
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
        Expr::Lit(mut expr) => {
            transform_literal(&mut expr);
            literal_to_gpu(expr)
        }
        Expr::Assign(expr) => assign_to_gpu(expr, module),
        Expr::Unary(expr) => unary_to_gpu(expr, module),
        Expr::Binary(expr) => binary_to_gpu(expr, module),
        Expr::If(expr) => Expr::Verbatim(if_to_gpu(expr, module)),
        Expr::While(expr) => Expr::Verbatim(while_to_gpu(expr, module).to_token_stream()),
        Expr::ForLoop(expr) => Expr::Verbatim(for_loop_to_gpu(expr, module).to_token_stream()),
        Expr::Break(expr) => break_to_gpu(expr, module).into(),
        Expr::Continue(expr) => continue_to_gpu(expr, module).into(),
        Expr::Range(expr) => range_to_gpu(expr, module),
        Expr::Array(expr) => array_to_gpu(expr, module),
        expr @ (Expr::Path(_)
        | Expr::Paren(_)
        | Expr::Call(_)
        | Expr::MethodCall(_)
        | Expr::Reference(_)
        | Expr::Index(_)
        | Expr::Field(_)
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

fn transform_literal(expr: &mut ExprLit) {
    if let Lit::Int(lit) = &mut expr.lit {
        if lit.suffix() == "u" {
            *lit = LitInt::new(&format!("{lit}32"), lit.span());
        }
    }
}

fn literal_to_gpu(value: impl ToTokens) -> Expr {
    parse_quote_spanned! { value.span() => ::ragna::Cpu::to_gpu(&#value) }
}

fn assign_to_gpu(expr: ExprAssign, module: &mut GpuModule) -> Expr {
    let span = expr.span();
    let attrs = &expr.attrs;
    let left = module.fold_expr(*expr.left);
    let right = module.fold_expr(*expr.right);
    parse_quote_spanned! { span => #(#attrs)* (::ragna::assign(#left, #right)) }
}

fn unary_to_gpu(mut expr: ExprUnary, module: &mut GpuModule) -> Expr {
    if let Expr::Lit(lit) = &mut *expr.expr {
        // to avoid out of range error with -2_147_483_648_i32 value
        transform_literal(lit);
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
                let #var_ident = ::ragna::create_uninit_var();
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

fn while_to_gpu(expr: ExprWhile, module: &mut GpuModule) -> Stmt {
    module.current_loop_level += 1;
    if let Some(label) = &expr.label {
        module
            .errors
            .push(syn::Error::new(label.span(), "labels not supported"));
    }
    let span = expr.span();
    let attrs = expr.attrs;
    let cond = module.fold_expr(*expr.cond);
    let cond_statements = mem::take(&mut module.extracted_statements);
    let body = module.fold_block(expr.body);
    module.current_loop_level -= 1;
    parse_quote_spanned! {
        span =>
        #(#attrs)*
        {
            ::ragna::loop_block();
            #(#cond_statements)*
            ::ragna::if_block(!#cond);
            ::ragna::break_();
            ::ragna::end_block();
            #body
            ::ragna::end_block();
        };
    }
}

fn for_loop_to_gpu(expr: ExprForLoop, module: &mut GpuModule) -> Stmt {
    if let Some(label) = &expr.label {
        module
            .errors
            .push(syn::Error::new(label.span(), "labels not supported"));
    }
    let span = expr.span();
    let attrs = expr.attrs;
    let iterable = expr.expr;
    let body = expr.body;
    match for_loop_mode(*expr.pat) {
        ForLoopMode::Value(value) => module.fold_stmt(parse_quote_spanned! {
            span =>
            {
                let __iterable = &(#iterable);
                let __index = 0u;
                let __len = ::ragna::Iterable::len(__iterable);
                #(#attrs)*
                while __index < __len {
                    let #value = ::ragna::Iterable::next(__iterable, __index);
                    #body;
                    __index += 1u;
                }
            };
        }),
        ForLoopMode::Enumerated(index, value) => module.fold_stmt(parse_quote_spanned! {
            span =>
            {
                let __iterable = &(#iterable);
                let __index = 0u;
                let __len = ::ragna::Iterable::len(__iterable);
                #(#attrs)*
                while __index < __len {
                    let #index = __index;
                    let #value = ::ragna::Iterable::next(__iterable, __index);
                    #body;
                    __index += 1u;
                }
            };
        }),
    }
}

fn for_loop_mode(pat: Pat) -> ForLoopMode {
    if let Pat::Tuple(pat) = pat {
        if pat.elems.len() == 2 {
            ForLoopMode::Enumerated(pat.elems[0].clone(), pat.elems[1].clone())
        } else {
            ForLoopMode::Value(Pat::Tuple(pat))
        }
    } else {
        ForLoopMode::Value(pat)
    }
}

enum ForLoopMode {
    Value(Pat),
    Enumerated(Pat, Pat),
}

fn break_to_gpu(expr: ExprBreak, module: &mut GpuModule) -> ExprCall {
    let span = expr.span();
    let attrs = expr.attrs;
    if module.current_loop_level == 0 {
        module
            .errors
            .push(syn::Error::new(span, "not allowed outside loops"));
    }
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
    parse_quote_spanned! { span => #(#attrs)* ::ragna::break_() }
}

fn continue_to_gpu(expr: ExprContinue, module: &mut GpuModule) -> ExprCall {
    let span = expr.span();
    let attrs = expr.attrs;
    if module.current_loop_level == 0 {
        module
            .errors
            .push(syn::Error::new(span, "not allowed outside loops"));
    }
    if let Some(label) = expr.label {
        module
            .errors
            .push(syn::Error::new(label.span(), "labels not supported"));
    }
    parse_quote_spanned! { span => #(#attrs)* ::ragna::continue_() }
}

fn range_to_gpu(expr: ExprRange, module: &mut GpuModule) -> Expr {
    let initial_expr = expr.clone();
    let span = expr.span();
    let attrs = expr.attrs;
    let start = if let Some(expr) = expr.start {
        module.fold_expr(*expr)
    } else {
        module
            .errors
            .push(syn::Error::new(span, "missing bound start"));
        return initial_expr.into();
    };
    let end = if let Some(expr) = expr.end {
        module.fold_expr(*expr)
    } else {
        module
            .errors
            .push(syn::Error::new(span, "missing bound end"));
        return initial_expr.into();
    };
    if let RangeLimits::Closed(token) = expr.limits {
        module
            .errors
            .push(syn::Error::new(token.span(), "unsupported range type"));
    }
    parse_quote_spanned! { span => #(#attrs)* ::ragna::Range::new(#start, #end) }
}

fn array_to_gpu(mut expr: ExprArray, module: &mut GpuModule) -> Expr {
    for elem in &mut expr.elems {
        *elem = module.fold_expr(elem.clone());
    }
    parse_quote_spanned! { expr.span() => ::ragna::Array::new(#expr) }
}
