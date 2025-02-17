fn main() {}

#[ragna::gpu]
mod gpu {
    use ragna::I32;

    enum Enum {}

    fn run() {
        static LOCAL_GLOB: I32 = 0;
        let mut var: I32;
        var = 0;
        let _ = 0;
        loop {}
    }
}
