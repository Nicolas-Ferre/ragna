#![allow(clippy::lossy_float_literal)]

use ragna::App;

#[test]
pub fn use_structs() {
    let app = App::default().with_module(gpu::register).texture().run(1);
    assert_eq!(app.read(*gpu::CONSTANT), Some(3..5));
    assert_eq!(app.read(*gpu::STRUCT_VAL), Some(1..10));
    assert_eq!(app.read(*gpu::NESTED_STRUCT_VAL), Some((1..5)..(3..4)));
    assert_eq!(app.read(*gpu::USED_FIELD), Some(5.));
}

#[ragna::gpu]
mod gpu {
    use ragna::{Cpu, Range, F32, I32};
    use std::ops;

    const CONSTANT_VALUE: ops::Range<i32> = 3..5;

    pub(crate) static CONSTANT: Range<I32> = CONSTANT_VALUE.to_gpu();
    pub(crate) static STRUCT_VAL: Range<I32> = 0..10;
    #[allow(unused_parens)]
    pub(crate) static NESTED_STRUCT_VAL: Range<Range<I32>> = (1..2)..(3..4);
    pub(crate) static USED_FIELD: F32 = 0.;

    #[compute]
    fn run() {
        STRUCT_VAL.start = 1;
        NESTED_STRUCT_VAL.start.end = 5;
        *USED_FIELD = f32(NESTED_STRUCT_VAL.start.end);
    }

    extern "wgsl" {
        fn f32(value: I32) -> F32;
    }
}
