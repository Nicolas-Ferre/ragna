use ragna::App;

#[test]
pub fn assign_values() {
    let app = App::default().with_module(gpu::register).run(1);
    assert_eq!(app.read(gpu::EXTERN_POW_RETURN), Some(9.));
    assert_eq!(app.read(gpu::EXTERN_SQRT_RETURN), Some(4.));
    assert_eq!(app.read(gpu::CUSTOM_FN_RETURN), Some(20.));
    assert_eq!(app.read(gpu::CUSTOM_FN_INPUT_RETURN), Some(5.));
}

#[ragna::gpu]
pub(crate) mod gpu {
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
        pub(crate) fn pow(value: f32, exponent: f32) -> f32;
    }

    #[rustfmt::skip]
    extern {
         pub(crate) fn sqrt(input: f32) -> f32;
    }

    fn multiply(value: f32, factor: f32) -> f32 {
        value = value * factor;
        value
    }
}
