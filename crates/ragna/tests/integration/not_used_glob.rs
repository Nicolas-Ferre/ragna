use crate::not_used_glob::gpu::{register, UNUSED_GLOB};
use ragna::App;

#[test]
pub fn run_app_without_glob() {
    let app = App::default().with_module(register).run(1);
    assert_eq!(app.read(UNUSED_GLOB), None);
}

#[ragna::gpu]
mod gpu {
    pub(super) static UNUSED_GLOB: i32 = 0;
    static USED_GLOB: i32 = 0;

    #[compute]
    fn run() {
        USED_GLOB = 10;
    }
}
