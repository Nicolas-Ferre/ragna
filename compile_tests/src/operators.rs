fn main() {}

#[ragna::gpu]
mod gpu {
    const CONSTANT: u32 = 10;

    static INVALID_UNARY: u32 = !CONSTANT;
    static UNSUPPORTED_UNARY: u32 = *&CONSTANT;
    static INVALID_BINARY: u32 = CONSTANT && CONSTANT;
    static UNSUPPORTED_BINARY: u32 = CONSTANT & CONSTANT;
}
