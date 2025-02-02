use crate::assignment::gpu::{register, FROM_CONSTANT, FROM_GLOB, FROM_MODIFIED_VAR, FROM_VAR};
use ragna::App;

#[test]
pub fn assign_values() {
    let app = App::default().with_module(register).run(1);
    assert_eq!(app.read(FROM_VAR), Some(10));
    assert_eq!(app.read(FROM_MODIFIED_VAR), Some(20));
    assert_eq!(app.read(FROM_CONSTANT), Some(30));
    assert_eq!(app.read(FROM_GLOB), Some(30));
}

mod gpu {
    use ragna::{App, Const, Gpu, GpuContext, Mut};

    const CONSTANT: Gpu<i32, Const> = Gpu::constant(30);

    pub(super) const FROM_VAR: Gpu<i32, Mut> =
        Gpu::glob("", 0, |ctx| Gpu::constant(0).extract(ctx));
    pub(super) const FROM_MODIFIED_VAR: Gpu<i32, Mut> =
        Gpu::glob("", 1, |ctx| Gpu::constant(0).extract(ctx));
    pub(super) const FROM_CONSTANT: Gpu<i32, Mut> =
        Gpu::glob("", 2, |ctx| Gpu::constant(0).extract(ctx));
    pub(super) const FROM_GLOB: Gpu<i32, Mut> =
        Gpu::glob("", 3, |ctx| Gpu::constant(0).extract(ctx));

    #[allow(const_item_mutation)]
    fn run(ctx: &mut GpuContext) {
        const LOCAL_GLOB: Gpu<i32, Mut> = Gpu::glob("", 4, |ctx| Gpu::constant(0).extract(ctx));
        let var = Gpu::var(ctx, Gpu::constant(10));
        FROM_VAR.assign(ctx, var);
        let mut modified_var = Gpu::var(ctx, Gpu::constant(10));
        modified_var.assign(ctx, Gpu::constant(20));
        FROM_MODIFIED_VAR.assign(ctx, modified_var);
        FROM_CONSTANT.assign(ctx, CONSTANT);
        FROM_GLOB.assign(ctx, FROM_CONSTANT);
        LOCAL_GLOB.assign(ctx, Gpu::constant(40));
    }

    pub(super) fn register(app: App) -> App {
        app.with_compute(run)
    }
}
