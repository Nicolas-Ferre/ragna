fn main() {}

#[ragna::gpu]
mod gpu {
    const CONSTANT: i32 = 30;

    fn run() {
        CONSTANT = 20;
    }
}
