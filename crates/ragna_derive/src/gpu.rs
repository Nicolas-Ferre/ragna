use proc_macro2::Span;
use proc_macro2::{Ident, TokenStream};
use quote::{quote, quote_spanned, ToTokens};
use std::ops::Deref;
use std::{iter, mem};
use syn::fold::Fold;
use syn::spanned::Spanned;
use syn::{
    fold, parse_quote, parse_quote_spanned, Attribute, Block, Expr, ExprUnary, Generics, Item,
    ItemConst, ItemFn, ItemMod, LitInt, Local, LocalInit, Meta, Pat, PatType, ReturnType, Stmt,
    Token, UnOp,
};

pub(crate) fn gpu(module: &ItemMod) -> TokenStream {
    let mut fold = GpuModule::default();
    let mut modified_module = fold.fold_item_mod(module.clone());
    if let Some((_, content)) = &mut modified_module.content {
        let compute_calls = fold
            .compute_fns
            .iter()
            .map(|fn_| quote_spanned! { fn_.span() => .with_compute(#fn_) });
        content.push(parse_quote! {
            #[allow(unreachable_pub)]
            pub fn register(app: ::ragna::App) -> ::ragna::App {
                app #(#compute_calls)*
            }
        });
    } else {
        unreachable!("not supported item");
    }
    let errors = fold.errors.into_iter().map(syn::Error::into_compile_error);
    quote! {
        #modified_module
        #(#errors)*
    }
}

#[derive(Default)]
struct GpuModule {
    next_id: u64,
    compute_fns: Vec<Ident>,
    errors: Vec<syn::Error>,
    extracted_statements: Vec<Stmt>,
}

impl GpuModule {
    fn next_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    fn is_compute_attribute(attr: &Attribute) -> bool {
        let path = attr.meta.path();
        matches!(attr.meta, Meta::Path(_))
            && path.segments.len() == 1
            && path.segments[0].ident == "compute"
    }

    fn new_var_ident(&mut self, span: Span) -> Ident {
        Ident::new(&format!("tmp{}", self.next_id()), span)
    }

    fn transform_literal(literal: impl ToTokens) -> Expr {
        parse_quote_spanned! {
            literal.span() =>
            ::ragna::Gpu::constant(#literal)
        }
    }

    fn transform_unary_op(&mut self, expr: ExprUnary, op_name: &str) -> Expr {
        let span = expr.span();
        let op_ident = Ident::new(op_name, span);
        let attrs = expr.attrs;
        let expr = self.fold_expr(*expr.expr);
        let var_ident = self.new_var_ident(span);
        self.extracted_statements.push(parse_quote_spanned! {
            span =>
            let #var_ident = #(#attrs)* ::ragna::Gpu::#op_ident(&#expr, __ctx);
        });
        parse_quote_spanned! { span => #var_ident }
    }
}

#[allow(clippy::wildcard_enum_match_arm)]
impl Fold for GpuModule {
    fn fold_block(&mut self, block: Block) -> Block {
        Block {
            brace_token: block.brace_token,
            stmts: block
                .stmts
                .into_iter()
                .flat_map(|stmt| {
                    let stmt = self.fold_stmt(stmt);
                    mem::take(&mut self.extracted_statements)
                        .into_iter()
                        .chain(iter::once(stmt))
                })
                .collect(),
        }
    }

    fn fold_expr(&mut self, expr: Expr) -> Expr {
        match expr {
            Expr::Lit(expr) => Self::transform_literal(expr),
            Expr::Assign(expr) => {
                let left = &expr.left;
                let right = self.fold_expr(expr.right.deref().clone());
                parse_quote_spanned! {
                    expr.span() =>
                    ::ragna::Gpu::assign(&mut #left, __ctx, #right)
                }
            }
            Expr::Unary(expr) => {
                if matches!(*expr.expr, Expr::Lit(_)) {
                    Self::transform_literal(expr)
                } else {
                    match &expr.op {
                        UnOp::Not(_) => self.transform_unary_op(expr, "not"),
                        UnOp::Neg(_) => self.transform_unary_op(expr, "neg"),
                        UnOp::Deref(_) | _ => {
                            self.errors
                                .push(syn::Error::new(expr.span(), "unsupported unary operator"));
                            fold::fold_expr_unary(self, expr).into()
                        }
                    }
                }
            }
            expr @ Expr::Path(_) => fold::fold_expr(self, expr),
            expr => {
                self.errors
                    .push(syn::Error::new(expr.span(), "unsupported expression"));
                fold::fold_expr(self, expr)
            }
        }
    }

    fn fold_item(&mut self, item: Item) -> Item {
        match item {
            Item::Static(item) => {
                let id = LitInt::new(&self.next_id().to_string(), item.span());
                let ty = item.ty;
                Item::Const(ItemConst {
                    attrs: item.attrs,
                    vis: item.vis,
                    const_token: Token![const](item.static_token.span),
                    ident: item.ident,
                    generics: Generics::default(),
                    colon_token: item.colon_token,
                    ty: parse_quote_spanned! {
                        ty.span() => ::ragna::Gpu<#ty, ::ragna::Mut>
                    },
                    eq_token: item.eq_token,
                    expr: {
                        let expr = self.fold_expr(*item.expr);
                        let statements = mem::take(&mut self.extracted_statements);
                        parse_quote_spanned! {
                            expr.span() => ::ragna::Gpu::glob(
                                module_path!(),
                                #id,
                                |__ctx|{ #(#statements)* ::ragna::Gpu::var(__ctx, #expr) }
                            )
                        }
                    },
                    semi_token: item.semi_token,
                })
            }
            item @ (Item::Const(_) | Item::Fn(_)) => fold::fold_item(self, item),
            item => {
                self.errors
                    .push(syn::Error::new(item.span(), "unsupported item"));
                fold::fold_item(self, item)
            }
        }
    }

    fn fold_item_const(&mut self, item: ItemConst) -> ItemConst {
        ItemConst {
            attrs: {
                let span = item.span();
                item.attrs
                    .into_iter()
                    .chain([parse_quote_spanned! {span => #[allow(unused_braces)]}])
                    .collect()
            },
            vis: item.vis,
            const_token: item.const_token,
            ident: item.ident,
            generics: item.generics,
            colon_token: item.colon_token,
            ty: {
                let ty = item.ty;
                parse_quote_spanned! {
                    ty.span() => ::ragna::Gpu<#ty, ::ragna::Const>
                }
            },
            eq_token: item.eq_token,
            expr: {
                let expr = self.fold_expr(*item.expr);
                let statements = mem::take(&mut self.extracted_statements);
                parse_quote_spanned! {
                    expr.span() => { #(#statements)* #expr }
                }
            },
            semi_token: item.semi_token,
        }
    }

    fn fold_item_fn(&mut self, mut item: ItemFn) -> ItemFn {
        if !item.sig.inputs.is_empty() {
            self.errors.push(syn::Error::new(
                item.sig.inputs.span(),
                "unsupported function params",
            ));
        }
        if let ReturnType::Type(_, ty) = &item.sig.output {
            self.errors
                .push(syn::Error::new(ty.span(), "unsupported function output"));
        }
        if item.attrs.iter().any(Self::is_compute_attribute) {
            self.compute_fns.push(item.sig.ident.clone());
        }
        let span = item.span();
        item.sig.inputs.insert(
            0,
            parse_quote_spanned! { item.sig.span() => __ctx: &mut ::ragna::GpuContext },
        );
        ItemFn {
            attrs: item
                .attrs
                .into_iter()
                .filter(|attr| !Self::is_compute_attribute(attr))
                .chain([
                    parse_quote_spanned! { span => #[allow(const_item_mutation, unused_braces)] },
                ])
                .collect(),
            vis: item.vis,
            sig: item.sig,
            block: Box::new(self.fold_block(item.block.deref().clone())),
        }
    }

    fn fold_local(&mut self, local: Local) -> Local {
        Local {
            attrs: local.attrs,
            let_token: local.let_token,
            pat: match local.pat {
                Pat::Type(pat) => {
                    let ty = pat.ty;
                    Pat::Type(PatType {
                        attrs: pat.attrs,
                        pat: pat.pat,
                        colon_token: pat.colon_token,
                        ty: parse_quote_spanned! { ty.span() => ::ragna::Gpu<#ty, ::ragna::Mut> },
                    })
                }
                pat @ Pat::Ident(_) => pat,
                _ => {
                    self.errors.push(syn::Error::new(
                        local.pat.span(),
                        "unsupported variable definition syntax",
                    ));
                    local.pat
                }
            },
            init: local.init.map(|init| self.fold_local_init(init)),
            semi_token: local.semi_token,
        }
    }

    fn fold_local_init(&mut self, init: LocalInit) -> LocalInit {
        let expr = self.fold_expr(init.expr.deref().clone());
        LocalInit {
            eq_token: init.eq_token,
            expr: parse_quote_spanned! {
                expr.span() => ::ragna::Gpu::var(__ctx, #expr)
            },
            diverge: init.diverge,
        }
    }
}
