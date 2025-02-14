use ragna::App;

#[test]
pub fn assign_values() {
    let app = App::default().with_module(gpu::register).run(1);
    assert_eq!(app.read(gpu::EXTERN_FN_RESULT), Some(9.));
    assert_eq!(app.read(gpu::EXTERN_GENERIC_FN_RESULT), Some(4.));
    assert_eq!(app.read(gpu::CUSTOM_FN_RESULT), Some(20.));
    assert_eq!(app.read(gpu::CUSTOM_GENERIC_FN_RESULT), Some(24.));
    assert_eq!(app.read(gpu::CUSTOM_FN_INPUT_RESULT), Some(5.));
}

#[ragna::gpu]
pub(crate) mod gpu {
    use ragna::GpuMul;

    pub(super) static EXTERN_FN_RESULT: f32 = pow(3., 2.);
    pub(super) static EXTERN_GENERIC_FN_RESULT: f32 = sqrt(16.);
    pub(super) static CUSTOM_FN_RESULT: f32 = 0.;
    pub(super) static CUSTOM_GENERIC_FN_RESULT: f32 = generic_multiply(12., 2.);
    pub(super) static CUSTOM_FN_INPUT_RESULT: f32 = 5.;

    #[compute]
    fn run() {
        let result = multiply(multiply(CUSTOM_FN_INPUT_RESULT, 2.), 2.);
        CUSTOM_FN_RESULT = result;
    }

    extern "wgsl" {
        pub(crate) fn pow(value: f32, exponent: f32) -> f32;
    }

    #[rustfmt::skip]
    extern {
         pub(crate) fn sqrt<T>(input: T) -> T where T: Clone;
    }

    fn multiply(value: f32, factor: f32) -> f32 {
        value = value * factor;
        value
    }

    fn generic_multiply<T>(value: T, factor: T) -> T
    where
        T: GpuMul<T, Output = T>,
    {
        value = value * factor;
        value
    }
}
