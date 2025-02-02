use crate::operations::{
    AssignVarOperation, CreateVarOperation, GpuConstant, GpuGlob, GpuValue, GpuVar, Operation,
};
use crate::types::GpuType;
use crate::GpuContext;
use derive_where::derive_where;
use std::any::{Any, TypeId};
use std::marker::PhantomData;

/// A tag to indicate a GPU value is mutable.
pub struct Mut;

/// A tag to indicate a GPU value is constant.
pub struct Const;

/// A GPU value.
#[derive_where(Debug, Clone, Copy; T)]
pub struct Gpu<T, K>
where
    T: GpuType,
{
    value: TypedGpuValue<T, K>,
    phantom: PhantomData<fn(K)>,
}

impl<T> Gpu<T, Const>
where
    T: GpuType,
{
    /// Creates a global constant.
    pub const fn constant(value: T) -> Self
    where
        T: ToString,
    {
        Self {
            value: TypedGpuValue::Constant(TypedGpuConstant { value }),
            phantom: PhantomData,
        }
    }
}

impl<T> Gpu<T, Mut>
where
    T: GpuType,
{
    // coverage: off (const fn)
    /// Creates a global variable stored in a GPU buffer.
    pub const fn glob(
        module: &'static str,
        id: u64,
        default_value: fn(&mut GpuContext) -> Self,
    ) -> Self {
        Self {
            value: TypedGpuValue::Glob(TypedGpuGlob {
                module,
                id,
                default_value,
            }),
            phantom: PhantomData,
        }
    }
    // coverage: on

    /// Creates a local variable.
    pub fn var(ctx: &mut GpuContext, value: Gpu<T, impl Any>) -> Self {
        ctx.register_type::<T>();
        let id = ctx.next_var_id();
        ctx.operations
            .push(Operation::CreateVar(CreateVarOperation {
                id,
                value: value.value.into(),
            }));
        Self {
            value: TypedGpuValue::Var(GpuVar { id }),
            phantom: PhantomData,
        }
    }

    /// Assigns a value
    pub fn assign(&mut self, ctx: &mut GpuContext, value: Gpu<T, impl Any>) {
        ctx.register_type::<T>();
        ctx.operations
            .push(Operation::AssignVar(AssignVarOperation {
                left_value: self.value.into(),
                right_value: value.value.into(),
            }));
    }
}

impl<T, K> Gpu<T, K>
where
    T: GpuType,
    K: 'static,
{
    /// Extract the value in a variable.
    pub fn extract(self, ctx: &mut GpuContext) -> Gpu<T, Mut>
    where
        T: ToString,
    {
        Gpu::var(ctx, self)
    }

    pub(crate) fn value(self) -> GpuValue {
        self.value.into()
    }
}

#[derive_where(Debug, Clone, Copy; T)]
enum TypedGpuValue<T, K>
where
    T: GpuType,
{
    Constant(TypedGpuConstant<T>),
    Glob(TypedGpuGlob<T, K>),
    Var(GpuVar),
}

impl<T, K> From<TypedGpuValue<T, K>> for GpuValue
where
    T: GpuType,
    K: 'static,
{
    fn from(value: TypedGpuValue<T, K>) -> Self {
        match value {
            TypedGpuValue::Constant(constant) => Self::Constant(GpuConstant {
                value: constant.value.into_wgsl(),
                type_id: TypeId::of::<T>(),
            }),
            TypedGpuValue::Glob(glob) => Self::Glob(GpuGlob {
                module: glob.module,
                id: glob.id,
                type_id: TypeId::of::<T>(),
                default_value: Box::new(move |ctx: &mut GpuContext| {
                    (glob.default_value)(ctx).value.into()
                }),
            }),
            TypedGpuValue::Var(var) => Self::Var(var),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct TypedGpuConstant<T>
where
    T: GpuType,
{
    value: T,
}

#[derive_where(Debug, Clone, Copy; T)]
struct TypedGpuGlob<T, K>
where
    T: GpuType,
{
    module: &'static str,
    id: u64,
    default_value: fn(&mut GpuContext) -> Gpu<T, K>,
}
