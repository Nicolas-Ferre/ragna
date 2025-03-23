fn main() {}

#[ragna::gpu]
mod gpu {
    use ragna::{F32, U32};

    fn func_with_invalid_param((a, b): (F32, F32)) {}
}
