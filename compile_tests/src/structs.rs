fn main() {}

#[ragna::gpu]
mod gpu {
    use ragna::U32;

    struct Unit;

    struct Empty {}

    struct Tuple(U32);

    struct WithLifetime<'a> {
        field: U32,
    }
}
