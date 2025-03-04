use crate::context::GpuContext;
use crate::operations::{ConstantAssignVarOperation, Field, GlobVar, Operation, Value, Var};
use crate::{context, Bool, Equal, U32};
use bitfield_struct::bitfield;
use std::any::TypeId;
use std::default::Default;
use std::marker::PhantomData;
use std::ops::Index;

pub(crate) mod array;
pub(crate) mod primitive;
pub(crate) mod range;

const MAX_NESTED_FIELDS: usize = 15;
const MAX_ITEM_ACCESS: usize = 50; // per shader

/// A trait implemented for Rust types that have a corresponding CPU type.
pub trait Cpu: Sized {
    /// The GPU type.
    type Gpu: Gpu;

    #[doc(hidden)]
    fn from_gpu(bytes: &[u8]) -> Self;

    #[doc(hidden)]
    fn to_wgsl(&self) -> String;

    /// Converts a value to a GPU value.
    fn to_gpu(&self) -> Self::Gpu {
        let var = crate::create_uninit_var::<Self::Gpu>();
        GpuContext::run_current(|ctx| {
            ctx.operations
                .push(Operation::ConstantAssignVar(ConstantAssignVarOperation {
                    left_value: var.value().into(),
                    right_value: self.to_wgsl(),
                }));
        });
        var
    }
}

/// A trait implemented for GPU types that have a corresponding CPU type.
pub trait Gpu: 'static + Copy {
    /// The CPU type.
    type Cpu: Cpu;

    #[doc(hidden)]
    fn details() -> GpuTypeDetails;

    #[doc(hidden)]
    fn value(self) -> GpuValue<Self>;

    #[doc(hidden)]
    fn unregistered() -> Self;

    #[doc(hidden)]
    fn from_value(value: GpuValue<Self>) -> Self;
}

// size of this type should be as small as possible to avoid stack overflow (e.g. with nested arrays)
#[doc(hidden)]
#[derive(Debug, Clone, Copy)]
pub struct GpuValue<T> {
    pub(crate) root: GpuValueRoot,
    pub(crate) extensions: [GpuValueExt; MAX_NESTED_FIELDS],
    phantom: PhantomData<fn(T)>,
}

impl<T> GpuValue<T> {
    pub fn glob(id: &'static &'static str) -> Self {
        Self {
            root: GpuValueRoot::Glob(id),
            extensions: [GpuValueExt::new(); MAX_NESTED_FIELDS],
            phantom: PhantomData,
        }
    }

    pub fn var(id: u32) -> Self {
        Self {
            root: GpuValueRoot::Var(id),
            extensions: [GpuValueExt::new(); MAX_NESTED_FIELDS],
            phantom: PhantomData,
        }
    }

    pub fn unregistered_var() -> Self {
        Self {
            root: GpuValueRoot::Var(context::next_var_id()),
            extensions: [GpuValueExt::new(); MAX_NESTED_FIELDS],
            phantom: PhantomData,
        }
    }

    fn field<U>(mut self, position: usize) -> GpuValue<U> {
        let next_field_id = self
            .extensions
            .iter()
            .enumerate()
            .find(|(_, ext)| ext.is_some())
            .map_or(0, |(index, _)| index + 1);
        let ext = self
            .extensions
            .get_mut(next_field_id)
            .expect("struct recursion limit reached");
        ext.set_id(position.try_into().expect("too many fields in struct"));
        ext.set_is_field(true);
        ext.set_is_some(true);
        GpuValue {
            root: self.root,
            extensions: self.extensions,
            phantom: PhantomData,
        }
    }

    // TODO: add index()
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum GpuValueRoot {
    Glob(&'static &'static str),
    Var(u32),
}

#[bitfield(u8)]
pub(crate) struct GpuValueExt {
    // either field position in the struct
    // or "local" ID (dedicated to array) of variable storing array index
    #[bits(6)]
    id: u8,
    #[bits(1)]
    is_field: bool,
    #[bits(1)]
    is_some: bool,
}

impl<T: 'static> From<GpuValue<T>> for Value {
    fn from(value: GpuValue<T>) -> Self {
        let type_id = TypeId::of::<T>();
        match value.root {
            GpuValueRoot::Glob(id) => {
                if value.extensions[0].is_some() {
                    Self::Field(Field {
                        source: Box::new(Self::Glob(GlobVar {
                            id,
                            type_id: TypeId::of::<()>(),
                        })),
                        positions: value
                            .extensions
                            .iter()
                            .take_while(|ext| ext.is_some())
                            .map(|ext| ext.id() as usize)
                            .collect(),
                        type_id,
                    })
                } else {
                    Self::Glob(GlobVar { id, type_id })
                }
            }
            GpuValueRoot::Var(id) => {
                if value.extensions[0].is_some() {
                    Self::Field(Field {
                        source: Box::new(Self::Var(Var {
                            id,
                            type_id: TypeId::of::<()>(),
                        })),
                        positions: value
                            .extensions
                            .iter()
                            .take_while(|ext| ext.is_some())
                            .map(|ext| ext.id() as usize)
                            .collect(),
                        type_id,
                    })
                } else {
                    Self::Var(Var { id, type_id })
                }
            }
        }
    }
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
    pub(crate) field_types: Vec<Self>,
}

impl GpuTypeDetails {
    pub(crate) fn size(&self) -> u64 {
        if let Some(size) = self.size {
            size
        } else {
            self.field_types.iter().map(Self::size).sum()
        }
    }
}

/// A trait implemented for GPU types on which it is possible to iterate (e.g. using a `for` loop).
pub trait Iterable: Index<U32> {
    /// The number of items contained in the iterable.
    fn len(&self) -> U32;

    /// Whether the iterable contains no item.
    fn is_empty(&self) -> Bool {
        <U32 as Equal<U32>>::apply(self.len(), 0_u32.to_gpu())
    }
}

#[derive(Clone, Copy)]
pub(crate) struct IndexItems<T>([T; MAX_ITEM_ACCESS]);

impl<T: Gpu> Default for IndexItems<T> {
    fn default() -> Self {
        let mut items = [T::unregistered(); MAX_ITEM_ACCESS];
        for item in &mut items {
            *item = T::unregistered();
        }
        Self(items)
    }
}

impl<T: Gpu> IndexItems<T> {
    fn next(&self, expr: T) -> &T {
        let item = GpuContext::run_current(|ctx| {
            let inner_index = ctx.next_index(self.0[0]);
            assert_ne!(inner_index, MAX_ITEM_ACCESS, "index call limit reached");
            ctx.register_var(self.0[inner_index]);
            &self.0[inner_index]
        });
        crate::assign(*item, expr);
        item
    }
}
