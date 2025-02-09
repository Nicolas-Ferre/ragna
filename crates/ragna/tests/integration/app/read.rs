use crate::app::read::gpu::{register, UNUSED_GLOB, USED_GLOB};
use ragna::{App, Gpu, GpuContext};

#[test]
pub fn read_uninitialized() {
    let app = App::default();
    assert_eq!(app.read(USED_GLOB), None);
}

#[test]
pub fn read_local_var() {
    let app = App::default().with_module(register).run(1);
    let var = Gpu::var(Gpu::constant(0), &mut GpuContext::default());
    assert_eq!(app.read(var), None);
}

#[test]
pub fn read_used_glob() {
    let app = App::default().with_module(register).run(1);
    assert_eq!(app.read(USED_GLOB), Some(10));
}

#[test]
pub fn read_unused_glob() {
    let app = App::default().with_module(register).run(1);
    assert_eq!(app.read(UNUSED_GLOB), None);
}

#[ragna::gpu]
mod gpu {
    pub(super) static UNUSED_GLOB: i32 = 0;
    pub(super) static USED_GLOB: i32 = 0;

    #[compute]
    fn run() {
        USED_GLOB = 10;
    }
}
