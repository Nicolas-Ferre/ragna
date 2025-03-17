use crate::context::GpuContext;
use crate::operations::{ConstantAssignVarOperation, Operation};
use crate::{Bool, Equal, U32};
use derive_where::derive_where;
use fxhash::FxHashMap;
use std::any::TypeId;

pub(crate) mod array;
pub(crate) mod primitive;
pub(crate) mod range;
pub(crate) mod vectors;

const MAX_NESTED_FIELDS: usize = 15;

/// A trait implemented for Rust types that have a corresponding CPU type.
pub trait Cpu: Sized {
    /// The GPU type.
    type Gpu: Gpu;

    #[doc(hidden)]
    fn from_gpu(bytes: &[u8]) -> Self;

    #[doc(hidden)]
    fn to_wgsl(&self) -> Wgsl;

    /// Converts a value to a GPU value.
    fn to_gpu(&self) -> Self::Gpu {
        let var = crate::create_uninit_var::<Self::Gpu>();
        let left_value = var.value();
        GpuContext::run_current(|ctx| {
            ctx.operations
                .push(Operation::ConstantAssignVar(ConstantAssignVarOperation {
                    left_value,
                    right_value: self.to_wgsl(),
                }));
        });
        var
    }
}

/// A trait implemented for GPU types that have a corresponding CPU type.
pub trait Gpu: 'static + Sync + Send + Copy {
    /// The CPU type.
    type Cpu: Cpu;

    #[doc(hidden)]
    fn details() -> GpuTypeDetails;

    #[doc(hidden)]
    fn value(self) -> GpuValue;

    #[doc(hidden)]
    fn from_value(value: GpuValue) -> Self;

    #[doc(hidden)]
    fn configure(self) -> Self {
        self
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub enum Wgsl {
    Value(String),
    Constructor(WgslConstructor),
}

#[doc(hidden)]
#[derive(Debug)]
pub struct WgslConstructor {
    pub type_id: TypeId,
    pub args: Vec<Wgsl>,
}

#[doc(hidden)]
#[derive(Debug, Clone, Copy)]
#[derive_where(PartialEq, Eq, Hash)]
pub struct GpuValue {
    #[derive_where(skip)]
    pub(crate) type_id: TypeId,
    pub(crate) root: GpuValueRoot,
    pub(crate) extensions: [GpuValueExt; MAX_NESTED_FIELDS],
}

impl GpuValue {
    #[doc(hidden)]
    pub fn field<T: Gpu>(self, position: u16) -> Self {
        self.extended::<T>(GpuValueExt::FieldPosition(position))
    }

    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub(crate) fn glob<T: Gpu>(id: &'static &'static str) -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            root: GpuValueRoot::Glob(id),
            extensions: [GpuValueExt::None; MAX_NESTED_FIELDS],
        }
    }

    pub(crate) fn var<T: Gpu>(id: u32) -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            root: GpuValueRoot::Var(id),
            extensions: [GpuValueExt::None; MAX_NESTED_FIELDS],
        }
    }

    pub(crate) fn vec_field<T: Gpu>(self, position: u8) -> Self {
        self.extended::<T>(GpuValueExt::VecFieldPosition(position))
    }

    pub(crate) fn index<T: Gpu>(self, index_var_id: u32) -> Self {
        self.extended::<T>(GpuValueExt::IndexVarId(index_var_id))
    }

    pub(crate) fn var_id(self) -> u32 {
        match self.root {
            GpuValueRoot::Glob(_) => unreachable!("internal error: value should be a local var"),
            GpuValueRoot::Var(id) => id,
        }
    }

    pub(crate) fn root_value<'a>(&self, globs: &'a [Self]) -> &'a Self {
        globs
            .iter()
            .find(|glob| glob.root == self.root)
            .expect("internal error: root value should be a glob")
    }

    fn extended<T: Gpu>(mut self, ext: GpuValueExt) -> Self {
        let updated_ext = self
            .extensions
            .iter_mut()
            .find(|ext| ext == &&GpuValueExt::None)
            .expect("struct recursion limit reached");
        *updated_ext = ext;
        Self {
            type_id: TypeId::of::<T>(),
            root: self.root,
            extensions: self.extensions,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum GpuValueRoot {
    Glob(&'static &'static str), // double reference to reduce size
    Var(u32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum GpuValueExt {
    FieldPosition(u16),
    VecFieldPosition(u8),
    IndexVarId(u32),
    None,
}

#[doc(hidden)]
#[derive(Debug, Clone)]
pub struct GpuTypeDetails {
    pub(crate) type_id: TypeId,
    // specified only if the name shouldn't be generated
    pub(crate) name: Option<&'static str>,
    // specified only for array type
    pub(crate) array_generics: Option<(Box<GpuTypeDetails>, usize)>,
    // specified only for native types
    pub(crate) size: Option<u64>,
    // specified only for native types
    pub(crate) alignment: Option<u64>,
    pub(crate) field_types: Vec<Self>,
}

impl GpuTypeDetails {
    #[doc(hidden)]
    pub fn new_struct<T: Gpu>(field_types: Vec<Self>) -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            name: None,
            array_generics: None,
            size: None,
            alignment: None,
            field_types,
        }
    }

    #[doc(hidden)]
    pub fn field_offset(&self, field_index: usize) -> u64 {
        if field_index == 0 {
            0
        } else if let Some(current_field) = self.field_types.get(field_index) {
            let previous_field = &self.field_types[field_index - 1];
            Self::round_up(
                current_field.alignment(),
                self.field_offset(field_index - 1) + previous_field.size(),
            )
        } else {
            self.size()
        }
    }

    pub(crate) fn from_fields(
        fields: &[GpuValue],
        types: &FxHashMap<TypeId, (usize, Self)>,
    ) -> Self {
        Self {
            type_id: TypeId::of::<()>(),
            name: None,
            array_generics: None,
            size: None,
            alignment: None,
            field_types: fields
                .iter()
                .map(|field| types[&field.type_id].1.clone())
                .collect(),
        }
    }

    pub(crate) fn size(&self) -> u64 {
        if let Some(size) = self.size {
            size
        } else {
            Self::round_up(
                self.alignment(),
                self.field_offset(self.field_types.len() - 1)
                    + self.field_types[self.field_types.len() - 1].size(),
            )
        }
    }

    pub(crate) fn alignment(&self) -> u64 {
        if let Some(alignment) = self.alignment {
            alignment
        } else {
            self.field_types
                .iter()
                .map(Self::alignment)
                .max()
                .expect("internal error: struct without field")
        }
    }

    pub(crate) fn round_up(k: u64, n: u64) -> u64 {
        n.div_ceil(k) * k
    }
}

/// A trait implemented for GPU types on which it is possible to iterate (e.g. using a `for` loop).
pub trait Iterable {
    /// The item type.
    type Item<'a>
    where
        Self: 'a;

    /// Returns the next item from the iteration index.
    fn next(&self, index: U32) -> Self::Item<'_>;

    /// The number of items contained in the iterable.
    fn len(&self) -> U32;

    /// Whether the iterable contains no item.
    fn is_empty(&self) -> Bool {
        <U32 as Equal<U32>>::apply(self.len(), 0_u32.to_gpu())
    }
}
