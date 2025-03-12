#![allow(clippy::lossy_float_literal)]

use ragna::{i32x2, i32x3, i32x4, App};

#[test]
pub fn use_vectors() {
    let app = App::default().with_module(gpu::register).run(1);
    assert_eq!(app.read(*gpu::NUM_FROM_CPU), Some(i32x2 { x: 2, y: 3 }));
    assert_eq!(app.read(*gpu::NUM_FROM_GPU), Some(i32x2 { x: 2, y: 3 }));
    assert_eq!(app.read(*gpu::VEC_X3), Some(i32x3 { x: 2, y: 3, z: 4 }));
    assert_eq!(
        app.read(*gpu::VEC_X4),
        Some(i32x4 {
            x: 2,
            y: 3,
            z: 4,
            w: 5
        })
    );
    assert_eq!(app.read(*gpu::X_VALUE), Some(2));
    assert_eq!(app.read(*gpu::Y_VALUE), Some(3));
    assert_eq!(app.read(*gpu::OPERATOR), Some(i32x2 { x: 12, y: 23 }));
}

#[ragna::gpu]
mod gpu {
    use ragna::{i32x2, Cpu, I32x2, I32x3, I32x4, I32};

    const CPU: i32x2 = i32x2 { x: 2, y: 3 };

    pub(super) static NUM_FROM_CPU: I32x2 = CPU.to_gpu();
    pub(super) static NUM_FROM_GPU: I32x2 = I32x2::new(2, 3);
    // To test different alignments
    pub(super) static VEC_X3: I32x3 = I32x3::new(2, 3, 4);
    pub(super) static VEC_X4: I32x4 = I32x4::new(2, 3, 4, 5);
    pub(super) static X_VALUE: I32 = 0;
    pub(super) static Y_VALUE: I32 = 1;
    pub(super) static OPERATOR: I32x2 = *NUM_FROM_CPU + I32x2::new(10, 20);

    #[compute]
    fn run() {
        *X_VALUE = NUM_FROM_CPU.x;
        *Y_VALUE = NUM_FROM_CPU.y;
    }
}
