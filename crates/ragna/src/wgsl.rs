use crate::operations::{Glob, Operation, Value};
use crate::types::{GpuType, GpuTypeDetails};
use crate::GpuContext;
use fxhash::FxHashMap;
use itertools::Itertools;
use std::any::TypeId;

const BUFFER_NAME: &str = "buffer";
const BUFFER_TYPE_NAME: &str = "Buffer";

pub(crate) fn header_code(globs: &[Glob], types: &FxHashMap<TypeId, GpuTypeDetails>) -> String {
    if globs.is_empty() {
        String::new()
    } else {
        format!(
            "@group(0) @binding(0)\nvar<storage, read_write> {}: {};\n\nstruct {} {{\n{}\n}}\n\n",
            BUFFER_NAME,
            BUFFER_TYPE_NAME,
            BUFFER_TYPE_NAME,
            globs
                .iter()
                .map(|glob| format!(
                    "    {}: {},",
                    glob_name(glob),
                    type_name(types, glob.type_id)
                ))
                .join("\n")
        )
    }
}

pub(crate) fn compute_shader_code(ctx: &GpuContext) -> String {
    format!(
        "@compute @workgroup_size(1, 1, 1)\nfn main() {{\n{}\n}}",
        ctx.operations
            .iter()
            .map(|operation| operation_code(ctx, operation, 4))
            .join("\n")
    )
}

fn operation_code(ctx: &GpuContext, operation: &Operation, indent: usize) -> String {
    match operation {
        Operation::CreateVar(op) => {
            format!(
                "{empty: >width$}var {} = {};",
                var_name(op.id),
                value_code(ctx, &op.value),
                empty = "",
                width = indent,
            )
        }
        Operation::AssignVar(op) => {
            format!(
                "{empty: >width$}{} = {};",
                value_code(ctx, &op.left_value),
                value_code(ctx, &op.right_value),
                empty = "",
                width = indent,
            )
        }
        Operation::Unary(op) => {
            let value = if op.value.value_type_id() == TypeId::of::<bool>() {
                format!("bool({})", value_code(ctx, &op.value))
            } else {
                value_code(ctx, &op.value)
            };
            let unary_expr = if op.var.value_type_id() == TypeId::of::<bool>() {
                let bool_gpu_type = bool::gpu_type_details().name;
                format!("{bool_gpu_type}({}{})", op.operator, value)
            } else {
                format!("{}{}", op.operator, value)
            };
            format!(
                "{empty: >width$}{} = {unary_expr};",
                value_code(ctx, &op.var),
                empty = "",
                width = indent,
            )
        }
    }
}

fn value_code(ctx: &GpuContext, value: &Value) -> String {
    match value {
        Value::Constant(constant) => {
            format!(
                "{}({})",
                type_name(&ctx.types, constant.type_id),
                constant.value
            )
        }
        Value::Glob(glob) => format!("{}.{}", BUFFER_NAME, glob_name(glob)),
        Value::Var(var) => format!("var{}", var.id),
    }
}

fn glob_name(glob: &Glob) -> String {
    format!("glob_{}_{}", glob.module.replace("::", "__"), glob.id)
}

fn type_name(types: &FxHashMap<TypeId, GpuTypeDetails>, type_id: TypeId) -> &str {
    types[&type_id].name
}

fn var_name(id: u64) -> String {
    format!("var{id}")
}
