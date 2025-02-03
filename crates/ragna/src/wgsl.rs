use crate::operations::{GpuGlob, GpuValue, Operation};
use crate::types::GpuTypeDetails;
use crate::GpuContext;
use fxhash::FxHashMap;
use itertools::Itertools;
use std::any::TypeId;

const BUFFER_NAME: &str = "buffer";
const BUFFER_TYPE_NAME: &str = "Buffer";

pub(crate) fn header_code(globs: &[GpuGlob], types: &FxHashMap<TypeId, GpuTypeDetails>) -> String {
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
    }
}

fn value_code(ctx: &GpuContext, value: &GpuValue) -> String {
    match value {
        GpuValue::Constant(constant) => {
            format!(
                "{}({})",
                type_name(&ctx.types, constant.type_id),
                constant.value
            )
        }
        GpuValue::Glob(glob) => format!("{}.{}", BUFFER_NAME, glob_name(glob)),
        GpuValue::Var(var) => format!("var{}", var.id),
    }
}

fn glob_name(glob: &GpuGlob) -> String {
    format!("glob_{}_{}", glob.module.replace("::", "__"), glob.id)
}

fn type_name(types: &FxHashMap<TypeId, GpuTypeDetails>, type_id: TypeId) -> &str {
    types[&type_id].name
}

fn var_name(id: u64) -> String {
    format!("var{id}")
}
