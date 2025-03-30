use ragna::App;

#[test]
pub fn use_functions() {
    let app = App::default()
        .with_module(gpu::register)
        .texture((1, 1))
        .run(1);
    assert_eq!(app.read(*gpu::EXTERN_FN_RESULT), Some(9.));
    assert_eq!(app.read(*gpu::EXTERN_GENERIC_FN_RESULT), Some(4.));
    assert_eq!(app.read(*gpu::CUSTOM_FN_RESULT), Some(20.));
    assert_eq!(app.read(*gpu::CUSTOM_GENERIC_FN_RESULT), Some(24.));
    assert_eq!(app.read(*gpu::CUSTOM_FN_INPUT_RESULT), Some(5.));
}

#[ragna::gpu]
pub(crate) mod gpu {
    use ragna::{Gpu, F32};
    use std::ops::Mul;

    pub(super) static EXTERN_FN_RESULT: F32 = pow(3., 2.);
    pub(super) static EXTERN_GENERIC_FN_RESULT: F32 = sqrt(16.);
    pub(super) static CUSTOM_FN_RESULT: F32 = 0.;
    pub(super) static CUSTOM_GENERIC_FN_RESULT: F32 = generic_multiply(12., 2.);
    pub(super) static CUSTOM_FN_INPUT_RESULT: F32 = 5.;

    #[compute]
    fn run() {
        let result = multiply(multiply(*CUSTOM_FN_INPUT_RESULT, 2.), 2.);
        *CUSTOM_FN_RESULT = result;
    }

    extern "wgsl" {
        pub(crate) fn pow(value: F32, exponent: F32) -> F32;
    }

    #[rustfmt::skip]
    extern {
         pub(crate) fn sqrt<T>(input: T) -> T where T: Gpu;
    }

    fn multiply(value: F32, factor: F32) -> F32 {
        value = value * factor;
        value
    }

    fn generic_multiply<T>(value: T, factor: T) -> T
    where
        T: Gpu + Mul<T, Output = T>,
    {
        value = value * factor;
        value
    }
}
