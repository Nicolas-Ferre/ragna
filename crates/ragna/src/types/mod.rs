use crate::context::GpuContext;
use crate::operations::{ConstantAssignVarOperation, Field, Glob, Operation, Value, Var};
use crate::{context, Bool, Equal, U32};
use std::any::TypeId;
use std::default::Default;
use std::ops::Index;

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
    fn to_wgsl(self) -> String;

    /// Converts a value to a GPU value.
    fn to_gpu(self) -> Self::Gpu {
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
pub enum GpuValue<T> {
    Glob(&'static str, u32, fn() -> T),
    Var(u32),
    GlobField(&'static str, u32, FieldPath),
    VarField(u32, FieldPath),
}

impl<T> GpuValue<T> {
    pub(crate) fn unregistered_var() -> Self {
        Self::Var(context::next_var_id())
    }

    fn field<U>(self, position: usize) -> GpuValue<U> {
        match self {
            Self::Glob(module, id, _) => GpuValue::GlobField(module, id, FieldPath::new(position)),
            Self::Var(id) => GpuValue::VarField(id, FieldPath::new(position)),
            Self::GlobField(module, id, field) => {
                GpuValue::GlobField(module, id, field.new_nested(position))
            }
            Self::VarField(id, field) => GpuValue::VarField(id, field.new_nested(position)),
        }
    }
}

impl<T: 'static> From<GpuValue<T>> for Value {
    fn from(value: GpuValue<T>) -> Self {
        let type_id = TypeId::of::<T>();
        match value {
            GpuValue::Glob(module, id, _) => Self::Glob(Glob {
                module,
                id,
                type_id,
            }),
            GpuValue::Var(id) => Self::Var(Var { id, type_id }),
            GpuValue::GlobField(module, id, field) => Self::Field(Field {
                source: Box::new(Self::Glob(Glob {
                    module,
                    id,
                    type_id: TypeId::of::<()>(),
                })),
                positions: field.positions[0..field.level_count as usize]
                    .iter()
                    .map(|&position| position as usize)
                    .collect(),
                type_id,
            }),
            GpuValue::VarField(id, field) => Self::Field(Field {
                source: Box::new(Self::Var(Var {
                    id,
                    type_id: TypeId::of::<()>(),
                })),
                positions: field.positions[0..field.level_count as usize]
                    .iter()
                    .map(|&position| position as usize)
                    .collect(),
                type_id,
            }),
        }
    }
}

#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FieldPath {
    level_count: u8,
    positions: [u8; MAX_NESTED_FIELDS],
}

impl FieldPath {
    fn new(position: usize) -> Self {
        let mut positions = <[u8; MAX_NESTED_FIELDS]>::default();
        positions[0] = u8::try_from(position).expect("too many fields in struct");
        Self {
            level_count: 1,
            positions,
        }
    }

    fn new_nested(mut self, position: usize) -> Self {
        assert_ne!(
            self.level_count as usize, MAX_NESTED_FIELDS,
            "struct recursion limit reached"
        );
        self.positions[self.level_count as usize] =
            u8::try_from(position).expect("too many fields in struct");
        self.level_count += 1;
        self
    }
}

#[doc(hidden)]
#[derive(Debug, Clone)]
pub struct GpuTypeDetails {
    pub(crate) type_id: TypeId,
    // specified only if the name shouldn't be generated
    pub(crate) name: Option<&'static str>,
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
