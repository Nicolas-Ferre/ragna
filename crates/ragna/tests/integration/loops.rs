use ragna::App;

#[test]
pub fn run_loops() {
    let app = App::default().with_module(gpu::register).run(1);
    assert_eq!(app.read(*gpu::WHILE_RESULT), Some(45));
    assert_eq!(app.read(*gpu::FOR_RESULT), Some(24));
    assert_eq!(app.read(*gpu::BREAK_RESULT), Some(15));
    assert_eq!(app.read(*gpu::CONTINUE_RESULT), Some(49));
}

#[ragna::gpu]
mod gpu {
    use ragna::{I32, U32};

    pub(super) static WHILE_RESULT: I32 = 0;
    pub(super) static FOR_RESULT: U32 = 0_u32;
    pub(super) static BREAK_RESULT: I32 = 0;
    pub(super) static CONTINUE_RESULT: I32 = 0;

    #[compute]
    fn run_while() {
        let i = 0;
        while i < 10 {
            *WHILE_RESULT += i;
            i += 1;
        }
    }

    #[compute]
    fn run_for() {
        for i in 3_u32..6_u32 {
            *FOR_RESULT += *i;
        }
        let range = &(3_u32..6_u32);
        for i in *range {
            *FOR_RESULT += *i;
        }
    }

    #[compute]
    fn run_break() {
        let i = 0;
        while i < 10 {
            *BREAK_RESULT += i;
            if i > 4 {
                break;
            }
            i += 1;
        }
    }

    #[compute]
    fn run_continue() {
        let i = 0;
        let continued = false;
        while i < 10 {
            *CONTINUE_RESULT += i;
            if i == 4 && !continued {
                continued = true;
                continue;
            }
            i += 1;
        }
    }
}
