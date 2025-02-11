use crate::fn_call::gpu::{register, EXTERN_POW_RETURN, EXTERN_SQRT_RETURN};
use ragna::App;

#[test]
pub fn assign_values() {
    let app = App::default().with_module(register).run(1);
    assert_eq!(app.read(EXTERN_POW_RETURN), Some(9.));
    assert_eq!(app.read(EXTERN_SQRT_RETURN), Some(4.));
}

#[ragna::gpu]
mod gpu {
    pub(super) static EXTERN_POW_RETURN: f32 = pow(3., 2.);
    pub(super) static EXTERN_SQRT_RETURN: f32 = sqrt(16.);

    extern "wgsl" {
        fn pow(value: f32, exponent: f32) -> f32;
    }

    #[rustfmt::skip]
    extern {
        fn sqrt(input: f32) -> f32;
    }
}
