use crate::multiple_runs::gpu::{register, GLOB};
use ragna::App;

#[test]
pub fn run_app_multiple_times() {
    let app = App::default().with_module(register).run(1);
    assert_eq!(app.read(GLOB), Some(10));
    let app = app.run(2);
    assert_eq!(app.read(GLOB), Some(10));
}

mod gpu {
    use ragna::{App, Gpu, GpuContext, Mut};

    pub(super) const GLOB: Gpu<i32, Mut> = Gpu::glob("", 0, |ctx| Gpu::constant(0).extract(ctx));

    #[allow(const_item_mutation)]
    fn run(ctx: &mut GpuContext) {
        GLOB.assign(ctx, Gpu::constant(10));
    }

    pub(super) fn register(app: App) -> App {
        app.with_compute(run)
    }
}
