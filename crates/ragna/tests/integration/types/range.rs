#![allow(clippy::lossy_float_literal)]

use ragna::App;

#[test]
pub fn use_ranges() {
    let app = App::default()
        .with_module(gpu::register)
        .texture((1, 1))
        .run(1);
    assert_eq!(app.read(*gpu::START), Some(3));
    assert_eq!(app.read(*gpu::END), Some(6));
    assert_eq!(app.read(*gpu::FIRST_ITEM), Some(3));
    assert_eq!(app.read(*gpu::SECOND_ITEM), Some(4));
    assert_eq!(app.read(*gpu::LEN_POSITIVE), Some(3));
    assert_eq!(app.read(*gpu::LEN_NEGATIVE), Some(0));
    assert_eq!(app.read(*gpu::IS_EMPTY_TRUE), Some(true));
    assert_eq!(app.read(*gpu::IS_EMPTY_FALSE), Some(false));
}

#[ragna::gpu]
mod gpu {
    use ragna::{Bool, Iterable, U32};

    pub(super) static START: U32 = 0u;
    pub(super) static END: U32 = 0u;
    pub(super) static FIRST_ITEM: U32 = 0u;
    pub(super) static SECOND_ITEM: U32 = 0u;
    pub(super) static LEN_POSITIVE: U32 = 0u;
    pub(super) static LEN_NEGATIVE: U32 = 42u;
    pub(super) static IS_EMPTY_TRUE: Bool = false;
    pub(super) static IS_EMPTY_FALSE: Bool = true;

    #[compute]
    fn run() {
        let range = 3u..6u;
        *START = range.start;
        *END = range.end;
        *FIRST_ITEM = range.next(0u);
        *SECOND_ITEM = range.next(1u);
        *LEN_POSITIVE = range.len();
        *LEN_NEGATIVE = (6u..3u).len();
        *IS_EMPTY_TRUE = (3u..3u).is_empty();
        *IS_EMPTY_FALSE = range.is_empty();
    }
}
