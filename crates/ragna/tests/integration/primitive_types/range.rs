#![allow(clippy::lossy_float_literal)]

use ragna::App;

#[test]
pub fn use_ranges() {
    let app = App::default().with_module(gpu::register).run(1);
    assert_eq!(app.read(*gpu::START), Some(3));
    assert_eq!(app.read(*gpu::END), Some(6));
    assert_eq!(app.read(*gpu::FIRST_ITEM), Some(3));
    assert_eq!(app.read(*gpu::SECOND_ITEM), Some(4));
    assert_eq!(app.read(*gpu::OUT_OF_BOUND_ITEM), Some(3));
    assert_eq!(app.read(*gpu::LEN), Some(3));
}

#[ragna::gpu]
mod gpu {
    use ragna::{Bool, Iterable, U32};

    pub(super) static START: U32 = 0_u32;
    pub(super) static END: U32 = 0_u32;
    pub(super) static FIRST_ITEM: U32 = 0_u32;
    pub(super) static SECOND_ITEM: U32 = 0_u32;
    pub(super) static OUT_OF_BOUND_ITEM: U32 = 0_u32;
    pub(super) static LEN: U32 = 0_u32;
    pub(super) static IS_EMPTY_TRUE: Bool = false;
    pub(super) static IS_EMPTY_FALSE: Bool = true;

    #[compute]
    fn run() {
        let range = 3_u32..6_u32;
        *START = range.start;
        *END = range.end;
        *FIRST_ITEM = range[0_u32];
        *SECOND_ITEM = range[1_u32];
        *OUT_OF_BOUND_ITEM = range[3_u32];
        *LEN = range.len();
        *IS_EMPTY_TRUE = (3_u32..3_u32).is_empty();
        *IS_EMPTY_FALSE = range.is_empty();
    }
}
