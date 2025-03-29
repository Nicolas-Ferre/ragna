use ragna::App;

#[test]
pub fn use_globs() {
    let app = App::default().with_module(gpu::register).testing().run(1);
    assert_eq!(app.read(*gpu::FROM_CONSTANT), Some(10));
    assert_eq!(app.read(*gpu::FROM_GLOB), Some(10));
}

#[ragna::gpu]
mod gpu {
    use ragna::{Cpu, I32};

    const CONSTANT: i32 = 10;

    pub(super) static FROM_CONSTANT: I32 = CONSTANT.to_gpu();
    pub(super) static FROM_GLOB: I32 = *FROM_CONSTANT;
}
