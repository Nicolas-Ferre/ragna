fn main() {}

#[ragna::gpu]
mod gpu {
    const CONSTANT: u32 = 10;

    static INVALID_NEG: u32 = -CONSTANT;
    static INVALID_NOT: u32 = !CONSTANT;
    static INVALID_DEREF: u32 = *&CONSTANT;
}
