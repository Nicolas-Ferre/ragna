use ragna::App;

#[test]
pub fn run_app_multiple_times() {
    let app = App::default()
        .with_module(gpu::register)
        .texture((1, 1))
        .run(1);
    assert_eq!(app.read(*gpu::GLOB), Some(10));
    let app = app.run(2);
    assert_eq!(app.read(*gpu::GLOB), Some(10));
}

#[ragna::gpu]
mod gpu {
    use ragna::I32;

    pub(crate) static GLOB: I32 = 0;

    #[compute]
    fn run() {
        *GLOB = 10;
    }
}
