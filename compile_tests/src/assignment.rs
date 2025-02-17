fn main() {}

#[ragna::gpu]
mod gpu {
    fn run() {
        let i32_value = 0_i32;
        i32_value = 1.;
    }
}
