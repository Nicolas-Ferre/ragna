use crate::gpu::GpuModule;
use syn::spanned::Spanned;
use syn::{
    parse_quote_spanned, GenericParam, Generics, TraitBound, TraitBoundModifier, TypeParamBound,
};

pub(crate) fn params_to_gpu(mut generics: Generics, module: &mut GpuModule) -> Generics {
    for param in &mut generics.params {
        match param {
            GenericParam::Type(ty) => ty.bounds.push(TypeParamBound::Trait(TraitBound {
                paren_token: None,
                modifier: TraitBoundModifier::None,
                lifetimes: None,
                path: parse_quote_spanned! { ty.span() => ::ragna::GpuType },
            })),
            param @ (GenericParam::Lifetime(_) | GenericParam::Const(_)) => {
                module
                    .errors
                    .push(syn::Error::new(param.span(), "unsupported generic param"));
            }
        }
    }
    generics
}
