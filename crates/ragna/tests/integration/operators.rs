#![allow(clippy::lossy_float_literal)]

use crate::operators::gpu::{
    register, ADD_VALUE, AND_FALSE_VALUE, AND_TRUE_VALUE, BOOL_NOT_VALUE, DIV_VALUE,
    EQ_FALSE_VALUE, EQ_TRUE_VALUE, F32_NEG_VALUE, GE_FALSE_VALUE, GE_TRUE_EQ_VALUE, GE_TRUE_VALUE,
    GLOB_UNARY_INIT_VALUE, GT_FALSE_EQ_VALUE, GT_FALSE_VALUE, GT_TRUE_VALUE, I32_DOUBLE_NEG_VALUE,
    I32_NEG_VALUE, LE_FALSE_VALUE, LE_TRUE_EQ_VALUE, LE_TRUE_VALUE, MUL_VALUE, NEQ_FALSE_VALUE,
    NEQ_TRUE_VALUE, OR_FALSE_VALUE, OR_TRUE_VALUE, REM_VALUE, SUB_VALUE,
};
use ragna::App;

#[test]
pub fn use_unary_operator() {
    let app = App::default().with_module(register).run(1);
    assert_eq!(app.read(I32_NEG_VALUE), Some(-10));
    assert_eq!(app.read(I32_DOUBLE_NEG_VALUE), Some(10));
    assert_eq!(app.read(F32_NEG_VALUE), Some(-20.));
    assert_eq!(app.read(BOOL_NOT_VALUE), Some(false));
    assert_eq!(app.read(GLOB_UNARY_INIT_VALUE), Some(-30));
}

#[test]
pub fn use_binary_operator() {
    let app = App::default().with_module(register).run(1);
    assert_eq!(app.read(ADD_VALUE), Some(3));
    assert_eq!(app.read(SUB_VALUE), Some(-1));
    assert_eq!(app.read(MUL_VALUE), Some(8));
    assert_eq!(app.read(DIV_VALUE), Some(2));
    assert_eq!(app.read(REM_VALUE), Some(1));
    assert_eq!(app.read(EQ_TRUE_VALUE), Some(true));
    assert_eq!(app.read(EQ_FALSE_VALUE), Some(false));
    assert_eq!(app.read(NEQ_TRUE_VALUE), Some(true));
    assert_eq!(app.read(NEQ_FALSE_VALUE), Some(false));
    assert_eq!(app.read(GT_TRUE_VALUE), Some(true));
    assert_eq!(app.read(GT_FALSE_VALUE), Some(false));
    assert_eq!(app.read(GT_FALSE_EQ_VALUE), Some(false));
    assert_eq!(app.read(GE_TRUE_VALUE), Some(true));
    assert_eq!(app.read(GE_TRUE_EQ_VALUE), Some(true));
    assert_eq!(app.read(GE_FALSE_VALUE), Some(false));
    assert_eq!(app.read(LE_TRUE_VALUE), Some(true));
    assert_eq!(app.read(LE_TRUE_EQ_VALUE), Some(true));
    assert_eq!(app.read(LE_FALSE_VALUE), Some(false));
    assert_eq!(app.read(AND_TRUE_VALUE), Some(true));
    assert_eq!(app.read(AND_FALSE_VALUE), Some(false));
    assert_eq!(app.read(OR_TRUE_VALUE), Some(true));
    assert_eq!(app.read(OR_FALSE_VALUE), Some(false));
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
    pub(super) static GT_TRUE_VALUE: bool = false;
    pub(super) static GT_FALSE_VALUE: bool = true;
    pub(super) static GT_FALSE_EQ_VALUE: bool = true;
    pub(super) static LT_TRUE_VALUE: bool = false;
    pub(super) static LT_FALSE_VALUE: bool = true;
    pub(super) static LT_FALSE_EQ_VALUE: bool = true;
    pub(super) static GE_TRUE_VALUE: bool = false;
    pub(super) static GE_TRUE_EQ_VALUE: bool = false;
    pub(super) static GE_FALSE_VALUE: bool = true;
    pub(super) static LE_TRUE_VALUE: bool = false;
    pub(super) static LE_TRUE_EQ_VALUE: bool = false;
    pub(super) static LE_FALSE_VALUE: bool = true;
    pub(super) static AND_TRUE_VALUE: bool = false;
    pub(super) static AND_FALSE_VALUE: bool = true;
    pub(super) static OR_TRUE_VALUE: bool = false;
    pub(super) static OR_FALSE_VALUE: bool = true;

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
        GT_TRUE_VALUE = 2 > 1;
        GT_FALSE_VALUE = 1 > 2;
        GT_FALSE_EQ_VALUE = 2 > 2;
        LT_TRUE_VALUE = 1 < 2;
        LT_FALSE_VALUE = 2 < 1;
        LT_FALSE_EQ_VALUE = 2 < 2;
        GE_TRUE_VALUE = 2 >= 1;
        GE_TRUE_EQ_VALUE = 2 >= 2;
        GE_FALSE_VALUE = 1 >= 2;
        LE_TRUE_VALUE = 1 <= 2;
        LE_TRUE_EQ_VALUE = 2 <= 2;
        LE_FALSE_VALUE = 2 <= 1;
        AND_TRUE_VALUE = true && true;
        AND_FALSE_VALUE = true && false;
        OR_TRUE_VALUE = true || false;
        OR_FALSE_VALUE = false || false;
    }
}
