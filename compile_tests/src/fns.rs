fn main() {}

#[ragna::gpu]
mod gpu {
    fn func_with_invalid_param_pattern((a, b): (f32, f32)) -> (f32, f32) {
        (a, b)
    }

    fn func_with_self(self) {}

    fn func_with_lifetime<'a, const N: usize>() {}
}
