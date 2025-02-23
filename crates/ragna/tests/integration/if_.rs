use ragna::App;

#[test]
pub fn run_if_else() {
    let app = App::default().with_module(gpu::register).run(1);
    assert_eq!(app.read(*gpu::IF_RESULT), Some(6));
    assert_eq!(app.read(*gpu::IF_EXPR_RESULT), Some(5));
    assert_eq!(app.read(*gpu::CONDITIONAL_RETURN_RESULT), Some(1));
}

#[ragna::gpu]
mod gpu {
    use ragna::I32;

    pub(super) static IF_RESULT: I32 = 0;
    pub(super) static IF_EXPR_RESULT: I32 = 0;
    pub(super) static CONDITIONAL_RETURN_RESULT: I32 = 0;

    #[compute]
    fn run_if() {
        let a = 0;
        let b = 3;
        if a < b {
            *IF_RESULT += 1;
        }
        if a > b {
            *IF_RESULT = 0;
        } else {
            *IF_RESULT += 2;
        }
        if a == b {
            *IF_RESULT = 0;
        } else if a < b {
            *IF_RESULT += 3;
        } else {
            *IF_RESULT = 0;
        }
    }

    #[compute]
    fn run_if_expr() {
        let value = if 0 < 3 {
            1
        } else if 0 == 3 {
            2
        } else {
            3
        };
        *IF_EXPR_RESULT = value + if 0 > 3 { 3 } else { 4 };
    }

    #[compute]
    fn run_conditional_return() {
        *CONDITIONAL_RETURN_RESULT = conditional_return();
    }

    fn conditional_return() -> I32 {
        if 0 < 3 {
            1
        } else {
            0
        }
    }
}
