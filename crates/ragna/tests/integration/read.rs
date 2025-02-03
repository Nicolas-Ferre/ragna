use crate::read::gpu::{register, GLOB};
use ragna::{App, Gpu, GpuContext};

#[test]
pub fn read_uninitialized() {
    let app = App::default();
    assert_eq!(app.read(GLOB), None);
}

#[test]
pub fn read_local_var() {
    let app = App::default().with_module(register).run(1);
    let var = Gpu::var(&mut GpuContext::default(), Gpu::constant(0));
    assert_eq!(app.read(var), None);
}

#[test]
pub fn read_glob() {
    let app = App::default().with_module(register).run(1);
    assert_eq!(app.read(GLOB), Some(42));
}

#[ragna::gpu]
mod gpu {
    pub(crate) static GLOB: i32 = 0;

    #[compute]
    fn run() {
        GLOB = 42;
    }
}
