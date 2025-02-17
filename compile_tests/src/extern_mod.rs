fn main() {}

#[ragna::gpu]
mod gpu {
    use ragna::{F32, U32};

    extern "C" {}

    extern "wgsl" {
        static STATIC: U32;

        fn no_return_type();

        fn func_with_self(self) -> F32;
    }
}
