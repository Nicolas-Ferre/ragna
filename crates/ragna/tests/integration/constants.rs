use ragna::App;

#[test]
pub fn use_constant() {
    let app = App::default().with_module(gpu::register).texture().run(1);
    assert_eq!(app.read(*gpu::CONSTANT_RES), Some(20));
}

#[ragna::gpu]
mod gpu {
    use ragna::{Cpu, I32};

    const CONSTANT: i32 = mul(2, 10);

    pub(super) static CONSTANT_RES: I32 = CONSTANT.to_gpu();

    const fn mul(lhs: i32, rhs: i32) -> i32 {
        let left = lhs;
        left * rhs
    }
}
