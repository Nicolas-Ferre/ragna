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
    assert_struct_eq(
        app.read(*gpu::FROM_GPU).unwrap(),
        gpu::test_struct_cpu(
            12,
            u32x3 {
                x: 20,
                y: 30,
                z: 40,
            },
            50.,
            60,
            [true, false, true],
        ),
    );
    assert_eq!(app.read(*gpu::FIELD_VALUE), Some(5.));
    assert_eq!(app.read(*gpu::VALUE_BEFORE_INCREMENT), Some(10));
    assert_eq!(app.read(*gpu::RETURNED_FIELD_VALUE), Some(50.));
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

    impl<T: Gpu, const N: usize> TestStruct<T, N> {
        fn increment_integer(&self) -> I32 {
            let old_value = self.integer;
            self.integer += 1;
            old_value
        }

        fn increment_static(value: &Self) {
            value.integer += 1;
        }

        fn float_internal_var_update(self, other: I32) -> F32 {
            let float = self.float;
            self.float += 1.; // shouldn't have impact outside the function
            other += 1; // shouldn't have impact outside the function
            float
        }
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
    pub(super) static FROM_GPU: TestStruct<U32, 3> = TestStruct::<U32, 3> {
        integer: 10,
        vector: U32x3::new(20u, 30u, 40u),
        float: 50.,
        custom: 60u,
        array: [true, false, true],
    };
    pub(super) static FIELD_VALUE: F32 = 0.;
    pub(super) static VALUE_BEFORE_INCREMENT: I32 = 0;
    pub(super) static RETURNED_FIELD_VALUE: F32 = 0.;

    #[compute]
    fn run() {
        *FIELD_VALUE = FROM_CPU.float;
        FROM_CPU.custom = 10u;
        *VALUE_BEFORE_INCREMENT = FROM_GPU.increment_integer();
        TestStruct::increment_static(&FROM_GPU);
        *RETURNED_FIELD_VALUE = FROM_GPU.float_internal_var_update(FROM_GPU.integer);
    }
}
