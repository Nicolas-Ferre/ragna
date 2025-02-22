#![allow(clippy::lossy_float_literal)]

use ragna::App;

#[test]
pub fn use_literals() {
    let app = App::default().with_module(gpu::register).run(1);
    assert_eq!(app.read(gpu::I32_POS_VALUE), Some(0x7FFF_FFFF));
    assert_eq!(app.read(gpu::I32_NEG_VALUE), Some(-0x8000_0000));
    assert_eq!(app.read(gpu::U32_VALUE), Some(0xFFFF_FFFF));
    assert_eq!(
        app.read(gpu::F32_INT_VALUE),
        Some(999_999_999_999_999_999_999.)
    );
    assert_eq!(app.read(gpu::F32_FRAC_VALUE), Some(123.456));
    assert_eq!(app.read(gpu::F32_NEG_VALUE), Some(-123.456));
    assert_eq!(app.read(gpu::BOOL_FALSE_VALUE), Some(false));
    assert_eq!(app.read(gpu::BOOL_TRUE_VALUE), Some(true));
    assert_eq!(app.read(gpu::BOOL_NOT_VALUE), Some(false));
}

#[ragna::gpu]
mod gpu {
    use ragna::{Bool, F32, I32, U32};

    pub(super) static I32_POS_VALUE: I32 = 0;
    pub(super) static I32_NEG_VALUE: I32 = 0;
    pub(super) static U32_VALUE: U32 = 0_u32;
    pub(super) static F32_INT_VALUE: F32 = 0.;
    pub(super) static F32_FRAC_VALUE: F32 = 0.;
    pub(super) static F32_NEG_VALUE: F32 = 0.;
    pub(super) static BOOL_FALSE_VALUE: Bool = false;
    pub(super) static BOOL_TRUE_VALUE: Bool = false;
    pub(super) static BOOL_NOT_VALUE: Bool = false;

    #[compute]
    #[allow(clippy::nonminimal_bool)]
    fn run() {
        I32_POS_VALUE = 0x7FFF_FFFF;
        I32_NEG_VALUE = -0x8000_0000;
        U32_VALUE = 0xFFFF_FFFF_u32;
        F32_INT_VALUE = 999_999_999_999_999_999_999.;
        F32_FRAC_VALUE = 123.456;
        F32_NEG_VALUE = -123.456;
        BOOL_FALSE_VALUE = false;
        BOOL_TRUE_VALUE = true;
        BOOL_NOT_VALUE = !true;
    }
}
