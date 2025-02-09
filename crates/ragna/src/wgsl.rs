use crate::operations::{Glob, Operation, Value};
use crate::types::{GpuType, GpuTypeDetails};
use crate::GpuContext;
use fxhash::FxHashMap;
use itertools::Itertools;
use std::any::TypeId;

const BUFFER_NAME: &str = "buf";
const BUFFER_TYPE_NAME: &str = "Buf";

pub(crate) fn header_code(types: &FxHashMap<TypeId, GpuTypeDetails>, globs: &[Glob]) -> String {
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
                    glob_name(glob, globs),
                    types[&glob.type_id].name
                ))
                .join("\n")
        )
    }
}

pub(crate) fn compute_shader_code(ctx: &GpuContext, globs: &[Glob]) -> String {
    format!(
        "@compute @workgroup_size(1, 1, 1)\nfn main() {{\n{}\n}}",
        ctx.operations
            .iter()
            .map(|operation| operation_code(operation, 4, globs))
            .join("\n")
    )
}

fn operation_code(operation: &Operation, indent: usize, globs: &[Glob]) -> String {
    match operation {
        Operation::DeclareVar(op) => {
            format!(
                "{empty: >width$}var {}: {};",
                var_name(op.id),
                op.type_.name,
                empty = "",
                width = indent,
            )
        }
        Operation::AssignVar(op) => {
            format!(
                "{empty: >width$}{} = {};",
                value_code(&op.left_value, globs),
                value_code(&op.right_value, globs),
                empty = "",
                width = indent,
            )
        }
        Operation::Unary(op) => {
            let value = if op.value.value_type_id() == TypeId::of::<bool>() {
                format!("bool({})", value_code(&op.value, globs))
            } else {
                value_code(&op.value, globs)
            };
            let unary_expr = if op.var.value_type_id() == TypeId::of::<bool>() {
                let bool_gpu_type = bool::gpu_type_details().name;
                format!("{bool_gpu_type}({}{})", op.operator, value)
            } else {
                format!("{}{}", op.operator, value)
            };
            format!(
                "{empty: >width$}{} = {unary_expr};",
                value_code(&op.var, globs),
                empty = "",
                width = indent,
            )
        }
    }
}

fn value_code(value: &Value, globs: &[Glob]) -> String {
    match value {
        Value::Constant(constant) => constant.value.clone(),
        Value::Glob(glob) => format!("{}.{}", BUFFER_NAME, glob_name(glob, globs)),
        Value::Var(var) => var_name(var.id),
    }
}

fn glob_name(glob: &Glob, globs: &[Glob]) -> String {
    let index = globs
        .iter()
        .position(|g| g == glob)
        .expect("internal error: glob not found");
    format!("g{index}")
}

fn var_name(id: u64) -> String {
    format!("v{id}")
}
