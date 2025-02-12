use ragna::App;

#[test]
pub fn run_app_multiple_times() {
    let app = App::default().with_module(gpu::register).run(1);
    assert_eq!(app.read(gpu::GLOB), Some(10));
    let app = app.run(2);
    assert_eq!(app.read(gpu::GLOB), Some(10));
}

#[ragna::gpu]
mod gpu {
    pub(crate) static GLOB: i32 = 0;

    #[compute]
    fn run() {
        GLOB = 10;
    }
}
