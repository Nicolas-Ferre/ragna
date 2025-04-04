use proc_macro2::{Ident, TokenStream};
use quote::{quote, quote_spanned};
use syn::fold::Fold;
use syn::spanned::Spanned;
use syn::{
    fold, parse_quote, Block, Expr, ExprReference, Item, ItemMod, Path, ReturnType, Signature,
    Stmt, Type, TypeReference,
};

mod attrs;
mod expressions;
mod fns;
mod foreign;
mod globs;
mod impls;
mod statements;
mod structs;
mod traits;
mod vars;

pub(crate) fn gpu(module: &ItemMod) -> TokenStream {
    let mut fold = GpuModule::default();
    let mut modified_module = fold.fold_item_mod(module.clone());
    if let Some((_, content)) = &mut modified_module.content {
        let globs = fold
            .globs
            .iter()
            .map(|glob| quote_spanned! { glob.span() => .with_glob(&#glob) });
        let compute_calls = fold
            .compute_fns
            .iter()
            .map(|fn_| quote_spanned! { fn_.span() => .with_compute(#fn_) });
        content.push(parse_quote! {
            #[allow(unreachable_pub)]
            pub fn register(app: ::ragna::App) -> ::ragna::App {
                app #(#globs)* #(#compute_calls)*
            }
        });
        content.extend(fold.generated_items);
    } else {
        unreachable!("not supported item");
    }
    let errors = fold.errors.into_iter().map(syn::Error::into_compile_error);
    quote! {
        #[allow(
            clippy::let_and_return,
            clippy::double_parens,
            clippy::unusual_byte_groupings,
        )]
        #modified_module
        #(#errors)*
    }
}

#[derive(Default)]
struct GpuModule {
    next_id: u64,
    globs: Vec<Ident>,
    compute_fns: Vec<Ident>,
    generated_items: Vec<Item>,
    errors: Vec<syn::Error>,
    extracted_statements: Vec<Stmt>,
    current_fn_signature: Option<Signature>,
    current_loop_level: u32,
}

impl GpuModule {
    fn next_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    fn is_current_fn_returning_copy(&self) -> bool {
        self.current_fn_signature
            .as_ref()
            .is_none_or(|sig| match &sig.output {
                ReturnType::Default => false,
                ReturnType::Type(_, ty) => !matches!(**ty, Type::Reference(_)),
            })
    }
}

impl Fold for GpuModule {
    fn fold_block(&mut self, block: Block) -> Block {
        statements::block_to_gpu(block, self)
    }

    fn fold_expr(&mut self, expr: Expr) -> Expr {
        expressions::expr_to_gpu(expr, self)
    }

    fn fold_expr_reference(&mut self, expr: ExprReference) -> ExprReference {
        if let Some(mutability) = expr.mutability {
            self.errors.push(syn::Error::new(
                mutability.span(),
                "`mut` keyword must not be used for references",
            ));
        }
        fold::fold_expr_reference(self, expr)
    }

    #[allow(clippy::wildcard_enum_match_arm)]
    fn fold_item(&mut self, item: Item) -> Item {
        match item {
            Item::Static(item) => globs::item_to_gpu(item, self).into(),
            Item::ForeignMod(item) => foreign::mod_to_gpu(item, self),
            Item::Fn(item) => fns::item_to_gpu(item, self).into(),
            Item::Struct(item) => Item::Verbatim(structs::item_to_gpu(item, self)),
            Item::Impl(item) => impls::block_to_gpu(item, self).into(),
            Item::Trait(item) => traits::item_to_gpu(item, self).into(),
            item @ (Item::Use(_) | Item::Const(_)) => item,
            item => {
                self.errors
                    .push(syn::Error::new(item.span(), "unsupported item"));
                fold::fold_item(self, item)
            }
        }
    }

    fn fold_path(&mut self, path: Path) -> Path {
        // paths must not be transformed (e.g. inner const generic values)
        path
    }

    fn fold_type_reference(&mut self, ty: TypeReference) -> TypeReference {
        if let Some(mutability) = ty.mutability {
            self.errors.push(syn::Error::new(
                mutability.span(),
                "`mut` keyword must not be used for references",
            ));
        }
        fold::fold_type_reference(self, ty)
    }
}
