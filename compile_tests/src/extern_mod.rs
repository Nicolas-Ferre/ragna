fn main() {}

#[ragna::gpu]
mod gpu {
    extern "C" {}

    extern "wgsl" {
        static STATIC: u32;
    }
}
