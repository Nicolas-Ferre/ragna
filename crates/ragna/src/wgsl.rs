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
        "@compute @workgroup_size(1, 1, 1)\nfn main() {{\n{}\n{}\n}}",
        globs
            .iter()
            .map(|glob| format!(
                // force the use of global variables to avoid pipeline creation error
                "    var _vg{} = {}.{};",
                glob.id,
                BUFFER_NAME,
                glob_name(glob, globs)
            ))
            .join("\n"),
        ctx.operations
            .iter()
            .map(|operation| operation_code(operation, globs))
            .join("\n")
    )
}

fn operation_code(operation: &Operation, globs: &[Glob]) -> String {
    match operation {
        Operation::DeclareVar(op) => {
            format!("    var {}: {};", var_name(op.id), op.type_.name)
        }
        Operation::AssignVar(op) => {
            format!(
                "    {} = {};",
                value_code(&op.left_value, globs),
                value_code(&op.right_value, globs),
            )
        }
        Operation::Unary(op) => {
            let value = function_arg(&op.value, globs);
            let operation = format!("{}{}", op.operator, value);
            let expr = returned_value(&op.var, operation);
            format!("    {} = {expr};", value_code(&op.var, globs))
        }
        Operation::Binary(op) => {
            let left_value = function_arg(&op.left_value, globs);
            let right_value = function_arg(&op.right_value, globs);
            let operation = format!("{} {} {}", left_value, op.operator, right_value);
            let expr = returned_value(&op.var, operation);
            format!("    {} = {expr};", value_code(&op.var, globs))
        }
    }
}

fn function_arg(value: &Value, globs: &[Glob]) -> String {
    if value.value_type_id() == TypeId::of::<bool>() {
        format!("bool({})", value_code(value, globs))
    } else {
        value_code(value, globs)
    }
}

fn returned_value(value: &Value, expr: String) -> String {
    if value.value_type_id() == TypeId::of::<bool>() {
        let bool_gpu_type = bool::gpu_type_details().name;
        format!("{bool_gpu_type}({expr})")
    } else {
        expr
    }
}

fn value_code(value: &Value, globs: &[Glob]) -> String {
    match value {
        Value::Constant(constant) => {
            format!("{}({})", constant.gpu_type.name, constant.value.clone())
        }
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
