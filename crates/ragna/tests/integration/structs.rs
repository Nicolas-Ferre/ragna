#![allow(clippy::lossy_float_literal)]

use ragna::App;

#[test]
pub fn use_structs() {
    let app = App::default().with_module(gpu::register).run(1);
    assert_eq!(app.read(*gpu::STRUCT_VAL), Some(1..10));
    assert_eq!(app.read(*gpu::NESTED_STRUCT_VAL), Some((1..5)..(3..4)));
}

#[ragna::gpu]
mod gpu {
    use ragna::{Range, I32};

    pub(crate) static STRUCT_VAL: Range<I32> = 0..10;
    #[allow(unused_parens)]
    pub(crate) static NESTED_STRUCT_VAL: Range<Range<I32>> = (1..2)..(3..4);

    #[compute]
    fn run() {
        STRUCT_VAL.start = 1;
        NESTED_STRUCT_VAL.start.end = 5;
    }
}
