use crate::operations::{
    AssignVarOperation, Constant, CreateVarOperation, Glob, Operation, Value, Var,
};
use crate::operators::{GpuNeg, GpuNot};
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
pub struct Gpu<T, M>
where
    T: GpuType,
{
    value: GpuValue<T, M>,
    phantom: PhantomData<fn(M)>,
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
            value: GpuValue::Constant(GpuConstant { value }),
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
            value: GpuValue::Glob(GpuGlob {
                module,
                id,
                default_value,
            }),
            phantom: PhantomData,
        }
    }
    // coverage: on

    /// Creates a local variable.
    pub fn var(value: Gpu<T, impl Any>, ctx: &mut GpuContext) -> Self {
        ctx.register_type::<T>();
        let id = ctx.next_var_id();
        ctx.operations
            .push(Operation::CreateVar(CreateVarOperation {
                id,
                value: value.value.into(),
            }));
        Self {
            value: GpuValue::Var(Var {
                id,
                type_id: TypeId::of::<T>(),
            }),
            phantom: PhantomData,
        }
    }

    /// Assigns a value
    pub fn assign(&mut self, value: Gpu<T, impl Any>, ctx: &mut GpuContext) {
        ctx.register_type::<T>();
        ctx.operations
            .push(Operation::AssignVar(AssignVarOperation {
                left_value: self.value.into(),
                right_value: value.value.into(),
            }));
    }
}

impl<T, M> Gpu<T, M>
where
    T: GpuType,
    M: 'static,
{
    /// Apply `-` unary operator on the value
    pub fn neg(&self, ctx: &mut GpuContext) -> Gpu<T, Mut>
    where
        T: GpuNeg,
    {
        T::apply(*self, ctx)
    }

    /// Apply `!` unary operator on the value
    pub fn not(&self, ctx: &mut GpuContext) -> Gpu<T, Mut>
    where
        T: GpuNot,
    {
        T::apply(*self, ctx)
    }

    pub(crate) fn value(self) -> Value {
        self.value.into()
    }
}

#[derive_where(Debug, Clone, Copy; T)]
enum GpuValue<T, M>
where
    T: GpuType,
{
    Constant(GpuConstant<T>),
    Glob(GpuGlob<T, M>),
    Var(Var),
}

impl<T, M> From<GpuValue<T, M>> for Value
where
    T: GpuType,
    M: 'static,
{
    fn from(value: GpuValue<T, M>) -> Self {
        match value {
            GpuValue::Constant(constant) => Self::Constant(Constant {
                value: constant.value.into_wgsl(),
                type_id: TypeId::of::<T>(),
            }),
            GpuValue::Glob(glob) => Self::Glob(Glob {
                module: glob.module,
                id: glob.id,
                type_id: TypeId::of::<T>(),
                default_value: Box::new(move |ctx: &mut GpuContext| {
                    (glob.default_value)(ctx).value.into()
                }),
            }),
            GpuValue::Var(var) => Self::Var(var),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct GpuConstant<T>
where
    T: GpuType,
{
    value: T,
}

#[derive_where(Debug, Clone, Copy; T)]
struct GpuGlob<T, M>
where
    T: GpuType,
{
    module: &'static str,
    id: u64,
    default_value: fn(&mut GpuContext) -> Gpu<T, M>,
}
