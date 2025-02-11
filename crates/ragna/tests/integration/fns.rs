use crate::fns::gpu::{
    register, CUSTOM_FN_INPUT_RETURN, CUSTOM_FN_RETURN, EXTERN_POW_RETURN, EXTERN_SQRT_RETURN,
};
use ragna::App;

#[test]
pub fn assign_values() {
    let app = App::default().with_module(register).run(1);
    assert_eq!(app.read(EXTERN_POW_RETURN), Some(9.));
    assert_eq!(app.read(EXTERN_SQRT_RETURN), Some(4.));
    assert_eq!(app.read(CUSTOM_FN_RETURN), Some(20.));
    assert_eq!(app.read(CUSTOM_FN_INPUT_RETURN), Some(5.));
}

#[ragna::gpu]
mod gpu {
    pub(super) static EXTERN_POW_RETURN: f32 = pow(3., 2.);
    pub(super) static EXTERN_SQRT_RETURN: f32 = sqrt(16.);
    pub(super) static CUSTOM_FN_RETURN: f32 = 0.;
    pub(super) static CUSTOM_FN_INPUT_RETURN: f32 = 5.;

    #[compute]
    fn run() {
        let result = multiply(multiply(CUSTOM_FN_INPUT_RETURN, 2.), 2.);
        CUSTOM_FN_RETURN = result;
    }

    extern "wgsl" {
        fn pow(value: f32, exponent: f32) -> f32;
    }

    #[rustfmt::skip]
    extern {
        fn sqrt(input: f32) -> f32;
    }

    fn multiply(value: f32, factor: f32) -> f32 {
        value = value * factor;
        value
    }
}
