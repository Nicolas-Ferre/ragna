#![allow(clippy::lossy_float_literal)]

use crate::types::structs::gpu::TestStructCpu;
use ragna::{u32x3, App, Gpu};
use std::fmt::Debug;

#[test]
pub fn use_structs() {
    let app = App::default().with_module(gpu::register).run(1);
    assert_struct_eq(
        app.read(*gpu::FROM_CPU).unwrap(),
        TestStructCpu {
            integer: 1,
            vector: u32x3 { x: 2, y: 3, z: 5 },
            float: 5.,
            custom: 10,
        },
    );
    assert_eq!(app.read(*gpu::FIELD_VALUE), Some(5.));
}

fn assert_struct_eq<T>(actual: TestStructCpu<T>, expected: TestStructCpu<T>)
where
    T: Gpu,
    T::Cpu: PartialEq + Debug,
{
    assert_eq!(actual.integer, expected.integer);
    assert_eq!(actual.vector, expected.vector);
    assert_eq!(actual.float, expected.float);
    assert_eq!(actual.custom, expected.custom);
}

#[ragna::gpu]
mod gpu {
    use ragna::{u32x3, Cpu, Gpu, U32x3, F32, I32, U32};

    #[allow(dead_code)]
    pub(super) struct TestStruct<T: Gpu> {
        pub(super) integer: I32,
        pub(super) vector: U32x3,
        pub(super) float: F32,
        pub(super) custom: T,
    }

    pub(super) const CPU: TestStructCpu<U32> = TestStructCpu {
        integer: 1,
        vector: u32x3 { x: 2, y: 3, z: 5 },
        float: 5.,
        custom: 6,
    };

    pub(super) static FROM_CPU: TestStruct<U32> = CPU.to_gpu();
    pub(super) static FIELD_VALUE: F32 = 0.;

    #[compute]
    fn run() {
        *FIELD_VALUE = FROM_CPU.float;
        FROM_CPU.custom = 10u;
    }
}
