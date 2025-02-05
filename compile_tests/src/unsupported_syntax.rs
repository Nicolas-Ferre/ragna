fn main() {}

#[ragna::gpu]
mod gpu {
    enum Enum {}

    fn run() {
        let mut var: i32;
        var = 0;
        let _ = 0;
        loop {}
    }

    fn invalid_fn(_param: i32) -> i32 {
        unimplemented!()
    }
}
