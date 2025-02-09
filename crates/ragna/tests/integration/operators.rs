#![allow(clippy::lossy_float_literal)]

use crate::operators::gpu::{
    register, ADD_VALUE, BOOL_NOT_VALUE, DIV_VALUE, EQ_FALSE_VALUE, EQ_TRUE_VALUE, F32_NEG_VALUE,
    GLOB_UNARY_INIT_VALUE, I32_DOUBLE_NEG_VALUE, I32_NEG_VALUE, MUL_VALUE, NEQ_FALSE_VALUE,
    NEQ_TRUE_VALUE, REM_VALUE, SUB_VALUE,
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
    assert_eq!(app.read(ADD_VALUE), Some(3));
    assert_eq!(app.read(SUB_VALUE), Some(-1));
    assert_eq!(app.read(MUL_VALUE), Some(8));
    assert_eq!(app.read(DIV_VALUE), Some(2));
    assert_eq!(app.read(REM_VALUE), Some(1));
    assert_eq!(app.read(EQ_TRUE_VALUE), Some(true));
    assert_eq!(app.read(EQ_FALSE_VALUE), Some(false));
    assert_eq!(app.read(NEQ_TRUE_VALUE), Some(true));
    assert_eq!(app.read(NEQ_FALSE_VALUE), Some(false));
}

#[ragna::gpu]
mod gpu {
    const CONSTANT: i32 = 30;

    pub(super) static I32_NEG_VALUE: i32 = 10;
    pub(super) static I32_DOUBLE_NEG_VALUE: i32 = 10;
    pub(super) static F32_NEG_VALUE: f32 = 20.;
    pub(super) static BOOL_NOT_VALUE: bool = true;
    pub(super) static GLOB_UNARY_INIT_VALUE: i32 = -CONSTANT;
    pub(super) static ADD_VALUE: i32 = 0;
    pub(super) static SUB_VALUE: i32 = 0;
    pub(super) static MUL_VALUE: i32 = 0;
    pub(super) static DIV_VALUE: i32 = 0;
    pub(super) static REM_VALUE: i32 = 0;
    pub(super) static EQ_TRUE_VALUE: bool = false;
    pub(super) static EQ_FALSE_VALUE: bool = true;
    pub(super) static NEQ_TRUE_VALUE: bool = false;
    pub(super) static NEQ_FALSE_VALUE: bool = true;

    #[compute]
    fn run() {
        I32_NEG_VALUE = -I32_NEG_VALUE;
        I32_DOUBLE_NEG_VALUE = --I32_DOUBLE_NEG_VALUE;
        F32_NEG_VALUE = -F32_NEG_VALUE;
        BOOL_NOT_VALUE = !BOOL_NOT_VALUE;
        let _tmp = GLOB_UNARY_INIT_VALUE;
        ADD_VALUE = 1 + 2;
        SUB_VALUE = 1 - 2;
        MUL_VALUE = 4 * 2;
        DIV_VALUE = 4 / 2;
        REM_VALUE = 5 % 2;
        REM_VALUE = 5 % 2;
        EQ_TRUE_VALUE = 2 == 2;
        EQ_FALSE_VALUE = 2 == 1;
        NEQ_TRUE_VALUE = 2 != 1;
        NEQ_FALSE_VALUE = 2 != 2;
    }
}
