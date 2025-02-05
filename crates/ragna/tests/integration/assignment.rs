use crate::assignment::gpu::{register, FROM_CONSTANT, FROM_GLOB, FROM_MODIFIED_VAR, FROM_VAR};
use ragna::App;

#[test]
pub fn assign_values() {
    let app = App::default().with_module(register).run(1);
    assert_eq!(app.read(FROM_VAR), Some(10));
    assert_eq!(app.read(FROM_MODIFIED_VAR), Some(20));
    assert_eq!(app.read(FROM_CONSTANT), Some(30));
    assert_eq!(app.read(FROM_GLOB), Some(30));
}

// TODO: add tests for static init from constant, from other glob, ...

#[ragna::gpu]
mod gpu {
    const CONSTANT: i32 = 30;

    pub(super) static FROM_VAR: i32 = 0;
    pub(super) static FROM_MODIFIED_VAR: i32 = 0;
    pub(super) static FROM_CONSTANT: i32 = 0;
    pub(super) static FROM_GLOB: i32 = 0;

    #[compute]
    fn run() {
        static LOCAL_GLOB: i32 = 0;
        let var: i32 = 10;
        FROM_VAR = var;
        let mut modified_var = 10;
        modified_var = 20;
        FROM_MODIFIED_VAR = modified_var;
        FROM_CONSTANT = CONSTANT;
        FROM_GLOB = FROM_CONSTANT;
        LOCAL_GLOB = 40;
    }
}
