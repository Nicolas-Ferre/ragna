#![allow(clippy::lossy_float_literal)]

use crate::operators::gpu::{
    register, BOOL_NOT_VALUE, F32_NEG_VALUE, GLOB_UNARY_INIT_VALUE, I32_DOUBLE_NEG_VALUE,
    I32_NEG_VALUE,
};
use ragna::App;

#[test]
pub fn assign_values() {
    let app = App::default().with_module(register).run(1);
    assert_eq!(app.read(I32_NEG_VALUE), Some(-10));
    assert_eq!(app.read(I32_DOUBLE_NEG_VALUE), Some(10));
    assert_eq!(app.read(F32_NEG_VALUE), Some(-20.));
    assert_eq!(app.read(BOOL_NOT_VALUE), Some(false));
    assert_eq!(app.read(GLOB_UNARY_INIT_VALUE), Some(-30));
}

#[ragna::gpu]
mod gpu {
    const CONSTANT: i32 = 30;

    pub(super) static I32_NEG_VALUE: i32 = 10;
    pub(super) static I32_DOUBLE_NEG_VALUE: i32 = 10;
    pub(super) static F32_NEG_VALUE: f32 = 20.;
    pub(super) static BOOL_NOT_VALUE: bool = true;
    pub(super) static GLOB_UNARY_INIT_VALUE: i32 = -CONSTANT;

    #[compute]
    fn run() {
        I32_NEG_VALUE = -I32_NEG_VALUE;
        I32_DOUBLE_NEG_VALUE = --I32_DOUBLE_NEG_VALUE;
        F32_NEG_VALUE = -F32_NEG_VALUE;
        BOOL_NOT_VALUE = !BOOL_NOT_VALUE;
        let _tmp = GLOB_UNARY_INIT_VALUE;
    }
}
