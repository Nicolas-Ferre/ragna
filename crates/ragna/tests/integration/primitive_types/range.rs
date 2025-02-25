#![allow(clippy::lossy_float_literal)]

use ragna::App;

#[test]
pub fn use_ranges() {
    let app = App::default().with_module(gpu::register).run(1);
    assert_eq!(app.read(*gpu::START), Some(3));
    assert_eq!(app.read(*gpu::END), Some(6));
    assert_eq!(app.read(*gpu::FIRST_ITEM), Some(3));
    assert_eq!(app.read(*gpu::SECOND_ITEM), Some(4));
}

#[ragna::gpu]
mod gpu {
    use ragna::U32;

    pub(super) static START: U32 = 0_u32;
    pub(super) static END: U32 = 0_u32;
    pub(super) static FIRST_ITEM: U32 = 0_u32;
    pub(super) static SECOND_ITEM: U32 = 0_u32;

    #[compute]
    fn run() {
        let range = 3_u32..6_u32;
        *START = range.start;
        *END = range.end;
        *FIRST_ITEM = range[0_u32];
        *SECOND_ITEM = range[1_u32];
    }
}
