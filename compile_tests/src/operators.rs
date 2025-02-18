fn main() {}

#[ragna::gpu]
mod gpu {
    use ragna::{U32, Cpu};

    const CONSTANT: u32 = 10;

    static INVALID_UNARY: U32 = !CONSTANT.to_gpu();
    static UNSUPPORTED_UNARY: U32 = *&CONSTANT.to_gpu();
    static INVALID_BINARY: U32 = CONSTANT.to_gpu() && CONSTANT.to_gpu();
    static UNSUPPORTED_BINARY: U32 = CONSTANT.to_gpu() & CONSTANT.to_gpu();

    #[compute]
    fn run() {
        UNSUPPORTED_BINARY += true;
    }
}
