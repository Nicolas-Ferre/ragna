use crate::operations::{Glob, Operation, Value};
use crate::types::GpuTypeDetails;
use crate::Bool;
use fxhash::FxHashMap;
use itertools::Itertools;
use std::any::TypeId;
use crate::context::GpuContext;

const BUFFER_NAME: &str = "buf";
const BUFFER_TYPE_NAME: &str = "Buf";
const STRUCT_NAME_PLACEHOLDER: &str = "<name>";

pub(crate) fn header_code(
    types: &FxHashMap<TypeId, (usize, GpuTypeDetails)>,
    globs: &[Glob],
) -> String {
    if globs.is_empty() {
        String::new()
    } else {
        let buffer_fields = globs
            .iter()
            .map(|glob| {
                let field_name = glob_name(glob, globs);
                let type_name = type_name(glob.type_id, types);
                format!("    {field_name}: {type_name},")
            })
            .join("\n");
        let structs = types
            .values()
            .filter(|(_, type_)| type_.name.is_none())
            .map(|(_, type_)| struct_(type_, types))
            .join("\n");
        format!(
            "@group(0) @binding(0)\nvar<storage, read_write> {BUFFER_NAME}: {BUFFER_TYPE_NAME};\n\n\
            struct {BUFFER_TYPE_NAME} {{\n{buffer_fields}\n}}\n\n\
            {structs}\n\n",
        )
    }
}

pub(crate) fn compute_shader_code(
    ctx: &GpuContext,
    types: &FxHashMap<TypeId, (usize, GpuTypeDetails)>,
    globs: &[Glob],
) -> String {
    format!(
        "@compute @workgroup_size(1, 1, 1)\nfn main() {{\n{}\n{}\n}}",
        globs
            .iter()
            .enumerate()
            .map(|(index, glob)| format!(
                // force the use of global variables to avoid pipeline creation error
                "    var _vg{} = {}.{};",
                index,
                BUFFER_NAME,
                glob_name(glob, globs)
            ))
            .join("\n"),
        ctx.operations
            .iter()
            .map(|operation| operation_code(operation, types, globs))
            .join("\n")
    )
}

fn struct_(
    type_details: &GpuTypeDetails,
    types: &FxHashMap<TypeId, (usize, GpuTypeDetails)>,
) -> String {
    let name = type_name(type_details.type_id, types);
    let fields = type_details
        .field_types
        .iter()
        .enumerate()
        .map(|(index, type_)| {
            let field_name = field_name(index);
            let type_name = type_name(type_.type_id, types);
            format!("    {field_name}: {type_name},")
        })
        .join("\n");
    format!("struct {name} {{\n{fields}\n}}")
}

fn operation_code(
    operation: &Operation,
    types: &FxHashMap<TypeId, (usize, GpuTypeDetails)>,
    globs: &[Glob],
) -> String {
    match operation {
        Operation::DeclareVar(op) => {
            let var_name = var_name(op.id);
            let type_name = type_name(op.type_.type_id, types);
            format!("    var {var_name}: {type_name};")
        }
        Operation::AssignVar(op) => {
            let left = value_code(&op.left_value, globs);
            let right = value_code(&op.right_value, globs);
            format!("    {left} = {right};")
        }
        Operation::ConstantAssignVar(op) => {
            let var_name = value_code(&op.left_value, globs);
            let type_name = type_name(op.left_value.value_type_id(), types);
            let value = op.right_value.replace(STRUCT_NAME_PLACEHOLDER, &type_name);
            format!("    {var_name} = {value};")
        }
        Operation::Unary(op) => {
            let var_name = value_code(&op.var, globs);
            let value = function_arg(&op.value, globs);
            let operation = format!("{}{value}", op.operator);
            let expr = returned_value(&op.var, operation, types);
            format!("    {var_name} = {expr};")
        }
        Operation::Binary(op) => {
            let var_name = value_code(&op.var, globs);
            let left_value = function_arg(&op.left_value, globs);
            let right_value = function_arg(&op.right_value, globs);
            let operation = format!("{} {} {}", left_value, op.operator, right_value);
            let expr = returned_value(&op.var, operation, types);
            format!("    {var_name} = {expr};")
        }
        Operation::FnCall(op) => {
            let var_name = value_code(&op.var, globs);
            let operation = format!(
                "{}({})",
                op.fn_name,
                op.args
                    .iter()
                    .map(|value| function_arg(value, globs))
                    .join(", ")
            );
            let expr = returned_value(&op.var, operation, types);
            format!("    {var_name} = {expr};")
        }
        Operation::IfBlock(op) => {
            let condition = value_code(&op.condition, globs);
            format!("    if (bool({condition})) {{")
        }
        Operation::ElseBlock => "    } else {".to_string(),
        Operation::LoopBlock => "    loop {".to_string(),
        Operation::EndBlock => "    }".to_string(),
        Operation::Break => "    break;".to_string(),
        Operation::Continue => "    continue;".to_string(),
    }
}

fn function_arg(value: &Value, globs: &[Glob]) -> String {
    if value.value_type_id() == TypeId::of::<Bool>() {
        format!("bool({})", value_code(value, globs))
    } else {
        value_code(value, globs)
    }
}

fn returned_value(
    value: &Value,
    expr: String,
    types: &FxHashMap<TypeId, (usize, GpuTypeDetails)>,
) -> String {
    let bool_type_id = TypeId::of::<Bool>();
    if value.value_type_id() == bool_type_id {
        let bool_gpu_type = type_name(bool_type_id, types);
        format!("{bool_gpu_type}({expr})")
    } else {
        expr
    }
}

fn value_code(value: &Value, globs: &[Glob]) -> String {
    match value {
        Value::Glob(glob) => format!("{}.{}", BUFFER_NAME, glob_name(glob, globs)),
        Value::Var(var) => var_name(var.id),
        Value::Field(field) => {
            let source = value_code(&field.source, globs);
            let fields = field
                .indexes
                .iter()
                .map(|index| field_name(*index))
                .join(".");
            format!("{source}.{fields}")
        }
    }
}

fn glob_name(glob: &Glob, globs: &[Glob]) -> String {
    format!("g{}", glob_index(glob, globs))
}

fn glob_index(glob: &Glob, globs: &[Glob]) -> usize {
    globs
        .iter()
        .position(|g| g == glob)
        .expect("internal error: glob not found")
}

fn var_name(id: u64) -> String {
    format!("v{id}")
}

fn type_name(type_id: TypeId, types: &FxHashMap<TypeId, (usize, GpuTypeDetails)>) -> String {
    let (id, details) = &types[&type_id];
    if let Some(name) = details.name {
        name.into()
    } else {
        format!("T{id}")
    }
}

fn field_name(field_index: usize) -> String {
    format!("f{field_index}")
}
