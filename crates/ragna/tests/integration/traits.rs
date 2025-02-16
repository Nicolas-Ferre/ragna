#![allow(clippy::lossy_float_literal)]

use ragna::{App, GpuType};

#[test]
pub fn assign_values() {
    let _app = App::default().with_module(gpu::register).run(1);
}

#[ragna::gpu]
mod gpu {
    pub(crate) trait TestTrait<E> {
        const CONSTANT: f32;

        type Type;
        type GenericType<T>;

        fn fn_with_self_param(self) -> f32;

        // fn fn_with_self_type_param(param: Self) -> f32;
        //
        // fn fn_with_associated_type_param(param: Self::Type) -> f32;
        //
        // fn fn_with_other_type_param(param: f32) -> f32;
        //
        // fn fn_with_returned_self_type(param: f32) -> Self;
        //
        // fn fn_with_returned_associated_type(param: f32) -> Self::Type;
        //
        // fn generic_fn_with_associated_type<T>(param: Self::GenericType<T>) -> Self::GenericType<T>;

        fn with_default_impl(param: f32) -> f32 {
            param
        }
    }

    // impl TestTrait<f32> for f32 {
    //     const CONSTANT: f32 = 0.0;
    //     type Type = ();
    //     type GenericType<T> = ();
    //
    //     fn fn_with_self_param(self) -> f32 {
    //         todo!()
    //     }
    // }

    fn f<T>(val: T) -> f32
    where
        T: TestTrait<f32>,
    {
        // val.fn_with_self_param()
        let a = 0.;
        let b = 0.;
        b
    }
}

#[ragna::gpu]
mod other {
    use crate::traits::gpu::TestTrait;

    fn f() {
        // TestTrait::<f32>::fn_with_self_param(1.0);
    }
}
