use crate::read::gpu::{register, GLOB};
use ragna::{App, Gpu};

#[test]
pub fn read_uninitialized() {
    let app = App::default();
    assert_eq!(app.read(GLOB), None);
}

#[test]
pub fn read_constant() {
    let app = App::default().with_module(register).run(1);
    assert_eq!(app.read(Gpu::constant(0)), None);
}

#[test]
pub fn read_glob() {
    let app = App::default().with_module(register).run(1);
    assert_eq!(app.read(GLOB), Some(42));
}

mod gpu {
    use ragna::{App, Gpu, GpuContext, Mut};

    pub(super) const GLOB: Gpu<i32, Mut> = Gpu::glob("", 0, |ctx| Gpu::constant(0).extract(ctx));

    #[allow(const_item_mutation)]
    fn run(ctx: &mut GpuContext) {
        GLOB.assign(ctx, Gpu::constant(42));
    }

    pub(super) fn register(app: App) -> App {
        app.with_compute(run)
    }
}
