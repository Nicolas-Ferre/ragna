#![allow(clippy::lossy_float_literal)]

use crate::types::structs::gpu::TestStructCpu;
use ragna::{u32x3, App, U32};

#[test]
pub fn use_structs() {
    let app = App::default().with_module(gpu::register).run(1);
    assert_struct_eq(
        app.read(*gpu::FROM_CPU).unwrap(),
        gpu::test_struct_cpu(1, u32x3 { x: 2, y: 3, z: 5 }, 5., 10, [false, true, false]),
    );
    assert_eq!(app.read(*gpu::FIELD_VALUE), Some(5.));
}

fn assert_struct_eq(actual: TestStructCpu<U32, 3>, expected: TestStructCpu<U32, 3>) {
    assert_eq!(actual.integer, expected.integer);
    assert_eq!(
        gpu::test_struct_vector(&actual),
        gpu::test_struct_vector(&expected)
    );
    assert_eq!(actual.float, expected.float);
    assert_eq!(actual.custom, expected.custom);
    assert_eq!(actual.array, expected.array);
}

#[ragna::gpu]
mod gpu {
    use ragna::{u32x3, Array, Bool, Cpu, Gpu, U32x3, F32, I32, U32};

    #[allow(dead_code)]
    pub(super) struct TestStruct<T: Gpu, const N: usize> {
        pub integer: I32,
        vector: U32x3,
        pub(super) float: F32,
        pub(super) custom: T,
        pub(super) array: Array<Bool, N>,
    }

    #[allow(dead_code)]
    struct StructWithOnlyPubFields {
        pub field: I32,
    }

    #[allow(dead_code)]
    struct StructWithOnlyRestrictedFields {
        pub(crate) field: I32,
    }

    #[allow(dead_code)]
    struct StructWithOnlyInheritedFields {
        field: I32,
    }

    pub(super) const fn test_struct_cpu(
        integer: i32,
        vector: u32x3,
        float: f32,
        custom: u32,
        array: [bool; 3],
    ) -> TestStructCpu<U32, 3> {
        TestStructCpu {
            integer,
            vector,
            float,
            custom,
            array,
        }
    }

    pub(super) const fn test_struct_vector(struct_: &TestStructCpu<U32, 3>) -> u32x3 {
        struct_.vector
    }

    pub(super) const CPU: TestStructCpu<U32, 3> = TestStructCpu {
        integer: 1,
        vector: u32x3 { x: 2, y: 3, z: 5 },
        float: 5.,
        custom: 6,
        array: [false, true, false],
    };

    pub(super) static FROM_CPU: TestStruct<U32, 3> = CPU.to_gpu();
    pub(super) static FIELD_VALUE: F32 = 0.;

    #[compute]
    fn run() {
        *FIELD_VALUE = FROM_CPU.float;
        FROM_CPU.custom = 10u;
    }
}
