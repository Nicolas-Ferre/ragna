fn main() {}

#[ragna::gpu]
mod gpu {
    use ragna::I32;

    fn func(param: &mut I32) -> &mut I32 {
        &mut 0
    }
}
