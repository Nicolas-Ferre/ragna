use ragna::App;

#[test]
pub fn use_imports() {
    let app = App::default().with_module(gpu::register).run(1);
    assert_eq!(app.read(*gpu::IMPORTED_FUNCTION), Some(9.));
    assert_eq!(app.read(*gpu::QUALIFIED_FUNCTION), Some(4.));
}

#[ragna::gpu]
mod gpu {
    use crate::fns::gpu::pow;
    use ragna::F32;

    pub(super) static IMPORTED_FUNCTION: F32 = pow(3., 2.);
    pub(super) static QUALIFIED_FUNCTION: F32 = crate::fns::gpu::sqrt(16.);
}
