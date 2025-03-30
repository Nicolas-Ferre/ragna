#![allow(missing_docs)]

use ragna::App;

fn main() {
    App::default()
        .with_module(gpu::register)
        .window()
        .run((0., 0., 1., 1.));
}

#[ragna::gpu]
mod gpu {
    use ragna::U32;

    static COUNTER: U32 = 0u;

    #[compute]
    fn increment() {
        *COUNTER += 1u;
    }
}
