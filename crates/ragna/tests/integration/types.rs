#![allow(clippy::lossy_float_literal)]

use crate::types::gpu::{
    register, BOOL_FALSE_VALUE, BOOL_TRUE_VALUE, F32_FRAC_VALUE, F32_INT_VALUE, I32_VALUE,
    U32_VALUE,
};
use ragna::App;

#[test]
pub fn assign_values() {
    let app = App::default().with_module(register).run(1);
    assert_eq!(app.read(I32_VALUE), Some(0x7FFF_FFFF));
    assert_eq!(app.read(U32_VALUE), Some(0xFFFF_FFFF));
    assert_eq!(app.read(F32_INT_VALUE), Some(999_999_999_999_999_999_999.));
    assert_eq!(app.read(F32_FRAC_VALUE), Some(123.456));
    assert_eq!(app.read(BOOL_FALSE_VALUE), Some(false));
    assert_eq!(app.read(BOOL_TRUE_VALUE), Some(true));
}

#[ragna::gpu]
mod gpu {
    pub(super) static I32_VALUE: i32 = 0;
    pub(super) static U32_VALUE: u32 = 0;
    pub(super) static F32_INT_VALUE: f32 = 0.;
    pub(super) static F32_FRAC_VALUE: f32 = 0.;
    pub(super) static BOOL_FALSE_VALUE: bool = false;
    pub(super) static BOOL_TRUE_VALUE: bool = false;

    #[compute]
    fn run() {
        I32_VALUE = 0x7FFF_FFFF;
        U32_VALUE = 0xFFFF_FFFF;
        F32_INT_VALUE = 999_999_999_999_999_999_999.;
        F32_FRAC_VALUE = 123.456;
        BOOL_FALSE_VALUE = false;
        BOOL_TRUE_VALUE = true;
    }
}
