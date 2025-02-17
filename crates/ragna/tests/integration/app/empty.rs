use ragna::App;

#[test]
pub fn run_empty_app() {
    App::default().run(1);
}

#[test]
pub fn run_app_with_empty_module() {
    App::default().with_module(no_item::register).run(1);
}

#[test]
pub fn run_app_with_no_used_glob() {
    let app = App::default().with_module(gpu::register).run(1);
    assert_eq!(app.read(gpu::GLOB), Some(10));
}

#[ragna::gpu]
mod no_item {}

#[ragna::gpu]
mod gpu {
    use ragna::I32;

    pub(crate) static GLOB: I32 = 10;

    #[compute]
    fn run() {
        let _var = 0_i32;
    }
}
