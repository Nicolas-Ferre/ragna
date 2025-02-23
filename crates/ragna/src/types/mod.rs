use crate::operations::{ConstantAssignVarOperation, Field, Glob, Operation, Value, Var};
use crate::GpuContext;
use std::any::TypeId;
use std::default::Default;

pub(crate) mod primitive;
pub(crate) mod range;

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
    fn from_value(value: GpuValue<Self>) -> Self;
}

#[doc(hidden)]
#[derive(Debug, Clone, Copy)]
pub enum GpuValue<T: Gpu> {
    Glob(&'static str, u64, fn() -> T),
    Var(u64),
    GlobField(&'static str, u64, FieldIndexes),
    VarField(u64, FieldIndexes),
}

impl<T: Gpu> GpuValue<T> {
    fn field<U: Gpu>(self, index: usize) -> GpuValue<U> {
        match self {
            Self::Glob(module, id, _) => GpuValue::GlobField(module, id, FieldIndexes::new(index)),
            Self::Var(id) => GpuValue::VarField(id, FieldIndexes::new(index)),
            Self::GlobField(module, id, field) => {
                GpuValue::GlobField(module, id, field.with_added_field(index))
            }
            Self::VarField(id, fields) => GpuValue::VarField(id, fields.with_added_field(index)),
        }
    }
}

impl<T: Gpu> From<GpuValue<T>> for Value {
    fn from(value: GpuValue<T>) -> Self {
        let type_id = TypeId::of::<T>();
        match value {
            GpuValue::Glob(module, id, _) => Self::Glob(Glob {
                module,
                id,
                type_id,
            }),
            GpuValue::Var(id) => Self::Var(Var { id, type_id }),
            GpuValue::GlobField(module, id, fields) => Self::Field(Field {
                source: Box::new(Self::Glob(Glob {
                    module,
                    id,
                    type_id: TypeId::of::<()>(),
                })),
                indexes: fields.indexes[0..fields.level_count].to_vec(),
                type_id,
            }),
            GpuValue::VarField(id, fields) => Self::Field(Field {
                source: Box::new(Self::Var(Var {
                    id,
                    type_id: TypeId::of::<()>(),
                })),
                indexes: fields.indexes[0..fields.level_count].to_vec(),
                type_id,
            }),
        }
    }
}

#[doc(hidden)]
#[derive(Debug, Clone, Copy)]
pub struct FieldIndexes {
    level_count: usize,
    indexes: [usize; 25],
}

impl FieldIndexes {
    fn new(index: usize) -> Self {
        let mut indexes = <[usize; 25]>::default();
        indexes[0] = index;
        Self {
            level_count: 1,
            indexes,
        }
    }

    fn with_added_field(mut self, index: usize) -> Self {
        assert_ne!(self.level_count, 25, "struct recursion limit reached");
        self.indexes[self.level_count] = index;
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
