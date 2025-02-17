fn main() {}

#[ragna::gpu]
mod gpu {
    use ragna::{F32, U32};

    fn func_with_self(self) {}
}
