use crate::globs::gpu::{register, FROM_CONSTANT, FROM_GLOB};
use ragna::App;

#[test]
pub fn assign_values() {
    let app = App::default().with_module(register).run(1);
    assert_eq!(app.read(FROM_CONSTANT), Some(10));
    assert_eq!(app.read(FROM_GLOB), Some(10));
}

#[ragna::gpu]
mod gpu {
    const CONSTANT: i32 = 10;

    pub(super) static FROM_CONSTANT: i32 = CONSTANT;
    pub(super) static FROM_GLOB: i32 = FROM_CONSTANT;

    #[compute]
    fn run() {
        let _from_constant = FROM_CONSTANT;
        let _from_glob = FROM_GLOB;
    }
}
