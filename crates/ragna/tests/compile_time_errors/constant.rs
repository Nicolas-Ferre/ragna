use ragna::{Const, Gpu, GpuContext};

fn main() {}

const CONSTANT: Gpu<i32, Const> = Gpu::constant(30);

fn run(ctx: &mut GpuContext) {
    CONSTANT.assign(ctx, Gpu::constant(20));
}
