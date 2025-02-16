use syn::spanned::Spanned;
use syn::{parse_quote_spanned, Type};

pub(crate) fn const_to_gpu(ty: &Type) -> Type {
    parse_quote_spanned! {
        ty.span() => ::ragna::Gpu<#ty, ::ragna::Const>
    }
}

pub(crate) fn mut_to_gpu(ty: &Type) -> Type {
    parse_quote_spanned! {
        ty.span() => ::ragna::Gpu<#ty, ::ragna::Mut>
    }
}

pub(crate) fn any_to_gpu(ty: &Type) -> Type {
    parse_quote_spanned! {
        ty.span() => ::ragna::Gpu<#ty, impl ::std::any::Any>
    }
}

pub(crate) fn is_self(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        type_path.qself.is_none()
            && type_path.path.segments.len() == 1
            && type_path.path.segments[0].ident == "Self"
    } else {
        false
    }
}
