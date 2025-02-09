#![allow(clippy::lossy_float_literal)]

use crate::operators::gpu::{
    register, ADD_ASSIGN_VALUE, ADD_VALUE, AND_FALSE_VALUE, AND_TRUE_VALUE, BOOL_NOT_VALUE,
    DIV_ASSIGN_VALUE, DIV_VALUE, EQ_FALSE_VALUE, EQ_TRUE_VALUE, F32_NEG_VALUE, GE_FALSE_VALUE,
    GE_TRUE_EQ_VALUE, GE_TRUE_VALUE, GLOB_UNARY_INIT_VALUE, GT_FALSE_EQ_VALUE, GT_FALSE_VALUE,
    GT_TRUE_VALUE, I32_DOUBLE_NEG_VALUE, I32_NEG_VALUE, LE_FALSE_VALUE, LE_TRUE_EQ_VALUE,
    LE_TRUE_VALUE, MUL_ASSIGN_VALUE, MUL_VALUE, NEQ_FALSE_VALUE, NEQ_TRUE_VALUE, OR_FALSE_VALUE,
    OR_TRUE_VALUE, REM_ASSIGN_VALUE, REM_VALUE, SUB_ASSIGN_VALUE, SUB_VALUE,
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

#[test]
pub fn use_operator_assign() {
    let app = App::default().with_module(register).run(1);
    assert_eq!(app.read(ADD_ASSIGN_VALUE), Some(15));
    assert_eq!(app.read(SUB_ASSIGN_VALUE), Some(5));
    assert_eq!(app.read(MUL_ASSIGN_VALUE), Some(50));
    assert_eq!(app.read(DIV_ASSIGN_VALUE), Some(2));
    assert_eq!(app.read(REM_ASSIGN_VALUE), Some(1));
}

#[ragna::gpu]
mod gpu {
    const CONSTANT: i32 = 30;

    pub(super) static I32_NEG_VALUE: i32 = 10;
    pub(super) static I32_DOUBLE_NEG_VALUE: i32 = 10;
    pub(super) static F32_NEG_VALUE: f32 = 20.;
    pub(super) static BOOL_NOT_VALUE: bool = true;
    pub(super) static GLOB_UNARY_INIT_VALUE: i32 = -CONSTANT;
    pub(super) static ADD_VALUE: i32 = 1 + 2;
    pub(super) static SUB_VALUE: i32 = 1 - 2;
    pub(super) static MUL_VALUE: i32 = 4 * 2;
    pub(super) static DIV_VALUE: i32 = 4 / 2;
    pub(super) static REM_VALUE: i32 = 5 % 2;
    pub(super) static EQ_TRUE_VALUE: bool = 2 == 2;
    pub(super) static EQ_FALSE_VALUE: bool = 2 == 1;
    pub(super) static NEQ_TRUE_VALUE: bool = 2 != 1;
    pub(super) static NEQ_FALSE_VALUE: bool = 2 != 2;
    pub(super) static GT_TRUE_VALUE: bool = 2 > 1;
    pub(super) static GT_FALSE_VALUE: bool = 1 > 2;
    pub(super) static GT_FALSE_EQ_VALUE: bool = 2 > 2;
    pub(super) static LT_TRUE_VALUE: bool = 1 < 2;
    pub(super) static LT_FALSE_VALUE: bool = 2 < 1;
    pub(super) static LT_FALSE_EQ_VALUE: bool = 2 < 2;
    pub(super) static GE_TRUE_VALUE: bool = 2 >= 1;
    pub(super) static GE_TRUE_EQ_VALUE: bool = 2 >= 2;
    pub(super) static GE_FALSE_VALUE: bool = 1 >= 2;
    pub(super) static LE_TRUE_VALUE: bool = 1 <= 2;
    pub(super) static LE_TRUE_EQ_VALUE: bool = 2 <= 2;
    pub(super) static LE_FALSE_VALUE: bool = 2 <= 1;
    pub(super) static AND_TRUE_VALUE: bool = true && true;
    pub(super) static AND_FALSE_VALUE: bool = true && false;
    pub(super) static OR_TRUE_VALUE: bool = true || false;
    pub(super) static OR_FALSE_VALUE: bool = false || false;
    pub(super) static ADD_ASSIGN_VALUE: i32 = 10;
    pub(super) static SUB_ASSIGN_VALUE: i32 = 10;
    pub(super) static MUL_ASSIGN_VALUE: i32 = 10;
    pub(super) static DIV_ASSIGN_VALUE: i32 = 10;
    pub(super) static REM_ASSIGN_VALUE: i32 = 10;

    #[compute]
    fn run() {
        I32_NEG_VALUE = -I32_NEG_VALUE;
        I32_DOUBLE_NEG_VALUE = --I32_DOUBLE_NEG_VALUE;
        F32_NEG_VALUE = -F32_NEG_VALUE;
        BOOL_NOT_VALUE = !BOOL_NOT_VALUE;
        let _tmp = GLOB_UNARY_INIT_VALUE;
        ADD_ASSIGN_VALUE += 5;
        SUB_ASSIGN_VALUE -= 5;
        MUL_ASSIGN_VALUE *= 5;
        DIV_ASSIGN_VALUE /= 5;
        REM_ASSIGN_VALUE %= 3;
    }
}
