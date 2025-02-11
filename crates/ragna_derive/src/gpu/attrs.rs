use syn::{Attribute, Meta};

pub(crate) fn is_compute(attr: &Attribute) -> bool {
    let path = attr.meta.path();
    matches!(attr.meta, Meta::Path(_))
        && path.segments.len() == 1
        && path.segments[0].ident == "compute"
}
