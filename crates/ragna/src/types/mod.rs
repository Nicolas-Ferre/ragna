use crate::context::GpuContext;
use crate::operations::{ConstantAssignVarOperation, Operation, Value, ValueExt};
use crate::{context, Bool, Equal, U32};
use array_init::array_init;
use bitfield_struct::bitfield;
use derive_where::derive_where;
use std::any::TypeId;
use std::default::Default;
use std::marker::PhantomData;
use std::ops::Index;

pub(crate) mod array;
pub(crate) mod primitive;
pub(crate) mod range;

const MAX_NESTED_FIELDS: usize = 15;
pub(crate) const MAX_INDEX_CALLS_PER_SHADER: usize = 50;

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
        let left_value = var.value().untyped();
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
pub trait Gpu: 'static + Copy {
    /// The CPU type.
    type Cpu: Cpu;

    #[doc(hidden)]
    fn details() -> GpuTypeDetails;

    #[doc(hidden)]
    fn value(self) -> GpuValue<Self>;

    #[doc(hidden)]
    fn from_value(value: GpuValue<Self>) -> Self;
}

// size of this type should be as small as possible to avoid stack overflow (e.g. with nested arrays)
#[doc(hidden)]
#[derive(Clone, Copy)]
#[derive_where(Debug)]
pub struct GpuValue<T> {
    pub(crate) root: GpuValueRoot,
    pub(crate) extensions: [GpuValueExt; MAX_NESTED_FIELDS],
    phantom: PhantomData<fn(T)>,
}

impl<T: Gpu> GpuValue<T> {
    pub(crate) fn glob(id: &'static &'static str) -> Self {
        Self {
            root: GpuValueRoot::Glob(id),
            extensions: [GpuValueExt::new(); MAX_NESTED_FIELDS],
            phantom: PhantomData,
        }
    }

    pub(crate) fn var(id: u32) -> Self {
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

    pub(crate) fn field<U>(self, position: usize) -> GpuValue<U> {
        let field_id = position.try_into().expect("too many fields in struct");
        self.extended(true, field_id)
    }

    pub(crate) fn index<U>(self, index_id: u8) -> GpuValue<U> {
        self.extended(false, index_id)
    }

    pub(crate) fn var_id(self) -> u32 {
        match self.root {
            GpuValueRoot::Glob(_) => unreachable!("internal error: value should be a local var"),
            GpuValueRoot::Var(id) => id,
        }
    }

    #[doc(hidden)]
    pub fn untyped(self) -> Value {
        self.untyped_with_option_ctx(None)
    }

    pub(crate) fn untyped_with_ctx(self, ctx: &GpuContext) -> Value {
        self.untyped_with_option_ctx(Some(ctx))
    }

    fn untyped_with_option_ctx(self, ctx: Option<&GpuContext>) -> Value {
        let mut value = Value {
            type_id: TypeId::of::<T>(),
            root: self.root,
            extensions: vec![],
        };
        for ext in self.extensions {
            if !ext.is_some() {
                break;
            }
            let untyped_ext = if ext.is_field() {
                ValueExt::FieldPosition(ext.id())
            } else if let Some(ctx) = ctx {
                ValueExt::IndexVarId(ctx.index(&value, ext.id()).value().var_id())
            } else {
                ValueExt::IndexVarId(GpuContext::run_current(|ctx| {
                    ctx.index(&value, ext.id()).value().var_id()
                }))
            };
            value.extensions.push(untyped_ext);
        }
        value
    }

    fn extended<U>(mut self, is_field: bool, id: u8) -> GpuValue<U> {
        let ext = self
            .extensions
            .iter_mut()
            .find(|ext| !ext.is_some())
            .expect("struct recursion limit reached");
        ext.set_id(id);
        ext.set_is_field(is_field);
        ext.set_is_some(true);
        GpuValue {
            root: self.root,
            extensions: self.extensions,
            phantom: PhantomData,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
    // `true` if this is a field, `false` if this is an indexing
    #[bits(1)]
    is_field: bool,
    #[bits(1)]
    is_some: bool,
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
pub(crate) struct IndexItems<T>([T; MAX_INDEX_CALLS_PER_SHADER]);

impl<T: Gpu> IndexItems<T> {
    fn new<P: Gpu>(parent: GpuValue<P>) -> Self {
        Self(array_init::<_, _, MAX_INDEX_CALLS_PER_SHADER>(|i| {
            T::from_value(parent.index(i.try_into().expect("internal error: out of bound index")))
        }))
    }

    fn next<P: Gpu>(&self, parent: P, index: U32) -> &T {
        let parent_value = parent.value().untyped();
        GpuContext::run_current(|ctx| &self.0[ctx.next_index_id(parent_value, index)])
    }
}
