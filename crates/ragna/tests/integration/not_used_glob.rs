use crate::not_used_glob::gpu::{register, UNUSED_GLOB};
use ragna::App;

#[test]
pub fn run_app_without_glob() {
    let app = App::default().with_module(register).run(1);
    assert_eq!(app.read(UNUSED_GLOB), None);
}

mod gpu {
    use ragna::{App, Gpu, GpuContext, Mut};

    pub(super) const UNUSED_GLOB: Gpu<i32, Mut> =
        Gpu::glob("", 0, |ctx| Gpu::constant(0).extract(ctx));
    const USED_GLOB: Gpu<i32, Mut> = Gpu::glob("", 1, |ctx| Gpu::constant(0).extract(ctx));

    #[allow(const_item_mutation)]
    fn run(ctx: &mut GpuContext) {
        USED_GLOB.assign(ctx, Gpu::constant(10));
    }

    pub(super) fn register(app: App) -> App {
        app.with_compute(run)
    }
}
