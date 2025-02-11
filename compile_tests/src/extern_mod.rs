fn main() {}

#[ragna::gpu]
mod gpu {
    extern "C" {}

    extern "wgsl" {
        static STATIC: u32;

        fn no_return_type();

        fn func_with_invalid_param_pattern((a, b): (f32, f32)) -> f32;

        fn func_with_self(self) -> f32;
    }
}
