fn main() {}

#[ragna::gpu]
mod gpu {
    fn func((a, b): (f32, f32)) -> (f32, f32) {
        (a, b)
    }
}
