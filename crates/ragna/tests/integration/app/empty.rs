use crate::app::empty::gpu::{register, GLOB};
use ragna::App;

#[test]
pub fn run_app_without_glob() {
    let app = App::default().with_module(register).run(1);
    assert_eq!(app.read(GLOB), None);
}

#[ragna::gpu]
mod gpu {
    pub(crate) static GLOB: i32 = 0;

    #[compute]
    fn run() {
        let _var = 0;
    }
}
