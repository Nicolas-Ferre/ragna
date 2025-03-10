use crate::context::GpuContext;
use crate::operations::{ConstantAssignVarOperation, Operation, Value, ValueExt};
use crate::{context, Bool, Equal, U32};
use array_init::array_init;
use bitfield_struct::bitfield;
use derive_where::derive_where;
use fxhash::FxHashMap;
use std::any::TypeId;
use std::default::Default;
use std::marker::PhantomData;

pub(crate) mod array;
pub(crate) mod primitive;
pub(crate) mod range;
pub(crate) mod vectors;

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
    #[allow(clippy::trivially_copy_pass_by_ref)]
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

    pub(crate) fn unregistered_var() -> Self {
        Self {
            root: GpuValueRoot::Var(context::next_var_id()),
            extensions: [GpuValueExt::new(); MAX_NESTED_FIELDS],
            phantom: PhantomData,
        }
    }

    pub(crate) fn field<U>(self, position: usize) -> GpuValue<U> {
        let field_id = position.try_into().expect("too many fields in struct");
        self.extended(GpuValueExtKind::Field, field_id)
    }

    pub(crate) fn vec_field<U>(self, position: usize) -> GpuValue<U> {
        let field_id = position.try_into().expect("too many fields in struct");
        self.extended(GpuValueExtKind::VecField, field_id)
    }

    pub(crate) fn index<U>(self, index_id: u8) -> GpuValue<U> {
        self.extended(GpuValueExtKind::Indexing, index_id)
    }

    pub(crate) fn var_id(self) -> u32 {
        match self.root {
            GpuValueRoot::Glob(_) => unreachable!("internal error: value should be a local var"),
            GpuValueRoot::Var(id) => id,
        }
    }

    #[doc(hidden)]
    pub fn untyped(self) -> Value {
        let mut value = Value {
            type_id: TypeId::of::<T>(),
            root: self.root,
            extensions: vec![],
        };
        for ext in self.extensions {
            let untyped_ext = match ext.kind() {
                GpuValueExtKind::None => break,
                GpuValueExtKind::Field => ValueExt::FieldPosition(ext.id()),
                GpuValueExtKind::VecField => ValueExt::FieldName(match ext.id() {
                    0 => "x",
                    1 => "y",
                    2 => "z",
                    _ => "w",
                }),
                GpuValueExtKind::Indexing => ValueExt::IndexVarId(GpuContext::run_current(|ctx| {
                    ctx.index(&value, ext.id()).value().var_id()
                })),
            };
            value.extensions.push(untyped_ext);
        }
        value
    }

    fn extended<U>(mut self, kind: GpuValueExtKind, id: u8) -> GpuValue<U> {
        let ext = self
            .extensions
            .iter_mut()
            .find(|ext| ext.kind() == GpuValueExtKind::None)
            .expect("struct recursion limit reached");
        ext.set_id(id);
        ext.set_kind(kind);
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
    #[bits(2)]
    kind: GpuValueExtKind,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum GpuValueExtKind {
    None = 0,
    Field = 1,
    VecField = 2,
    Indexing = 3,
}

impl GpuValueExtKind {
    const fn into_bits(self) -> u8 {
        self as _
    }

    const fn from_bits(value: u8) -> Self {
        match value {
            0 => Self::None,
            1 => Self::Field,
            2 => Self::VecField,
            _ => Self::Indexing,
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
    // specified only for native types
    pub(crate) alignment: Option<u64>,
    pub(crate) field_types: Vec<Self>,
}

impl GpuTypeDetails {
    pub(crate) fn from_fields(fields: &[Value], types: &FxHashMap<TypeId, (usize, Self)>) -> Self {
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

    pub(crate) fn field_offset(&self, field_index: usize) -> u64 {
        if field_index == 0 {
            0
        } else {
            let current_field = &self.field_types[field_index];
            let previous_field = &self.field_types[field_index - 1];
            Self::round_up(
                current_field.alignment(),
                self.field_offset(field_index - 1) + previous_field.size(),
            )
        }
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
        let index_value = index.value().untyped();
        GpuContext::run_current(|ctx| &self.0[ctx.next_index_id(parent_value, index_value)])
    }
}
