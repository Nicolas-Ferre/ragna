use ragna::{App, Cpu, I32};

#[test]
pub fn read_uninitialized() {
    let app = App::default();
    assert_eq!(app.read(gpu::USED_GLOB), None);
}

#[test]
pub fn read_used_glob() {
    let app = App::default().with_module(gpu::register).run(1);
    assert_eq!(app.read(gpu::USED_GLOB), Some(10));
}

#[test]
pub fn read_unused_glob() {
    let app = App::default().with_module(gpu::register).run(1);
    assert_eq!(app.read(gpu::UNUSED_GLOB), Some(20));
}

#[test]
pub fn read_not_registered_glob() {
    let app = App::default().with_module(gpu::register).run(1);
    let glob = I32::define_glob("", 0, || i32::to_gpu(0));
    assert_eq!(app.read(glob), None);
}

#[ragna::gpu]
mod gpu {
    use ragna::I32;

    pub(super) static UNUSED_GLOB: I32 = 20;
    pub(super) static USED_GLOB: I32 = 0;

    #[compute]
    fn run() {
        USED_GLOB = 10;
    }
}
