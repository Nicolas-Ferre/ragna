use ragna::App;

#[test]
pub fn assign_values() {
    let app = App::default().with_module(gpu::register).run(1);
    assert_eq!(app.read(gpu::FROM_CONSTANT), Some(10));
    assert_eq!(app.read(gpu::FROM_GLOB), Some(10));
}

#[ragna::gpu]
mod gpu {
    const CONSTANT: i32 = 10;

    pub(super) static FROM_CONSTANT: i32 = CONSTANT;
    pub(super) static FROM_GLOB: i32 = FROM_CONSTANT;
}
