#![allow(clippy::lossy_float_literal)]

use ragna::App;

#[test]
pub fn use_arrays() {
    let app = App::default().with_module(gpu::register).texture().run(1);
    assert_eq!(app.read(*gpu::FROM_CONSTANT), Some([1, 2, 3, 4]));
    assert_eq!(app.read(*gpu::FROM_ITEMS), Some([1, 2, 3, 4]));
    assert_eq!(app.read(*gpu::FROM_REPEATED), Some([42, 42, 42, 42]));
    assert_eq!(
        app.read(*gpu::NESTED),
        Some([[1, 2], [10, 11], [9, 6], [7, 8]])
    );
    assert_eq!(app.read(*gpu::DEEP), Some([[[[[10]]]]]));
    assert_eq!(app.read(*gpu::LENGTH), Some(4));
    assert_eq!(app.read(*gpu::FIRST_ITEM), Some(1));
    assert_eq!(app.read(*gpu::SECOND_ITEM), Some(2));
    assert_eq!(app.read(*gpu::OUT_OF_BOUND_ITEM), Some(1));
    assert_eq!(app.read(*gpu::NESTED_ARRAY_ITEM), Some(3));
    assert_eq!(
        app.read(*gpu::ITER_SUM),
        Some(1 + 2 + 10 + 11 + 9 + 6 + 7 + 8)
    );
    assert_eq!(
        app.read(*gpu::ENUMERATED_ITER_SUM),
        Some(1 + 2 + 10 + 11 + 9 + 6 + 7 + 8 + 4)
    );
}

#[ragna::gpu]
mod gpu {
    use ragna::{Array, Cpu, Iterable, U32};

    const CONSTANT: [u32; 4] = [1, 2, 3, 4];

    pub(super) static FROM_CONSTANT: Array<U32, 4> = CONSTANT.to_gpu();
    pub(super) static FROM_ITEMS: Array<U32, 4> = [1u, 2u, 3u, 4u];
    pub(super) static FROM_REPEATED: Array<U32, 4> = [42u; 4];
    pub(super) static NESTED: Array<Array<U32, 2>, 4> = [[1u, 2u], [3u, 4u], [5u, 6u], [7u, 8u]];
    #[allow(clippy::type_complexity)]
    pub(super) static DEEP: Array<Array<Array<Array<Array<U32, 1>, 1>, 1>, 1>, 1> = [[[[[0u]]]]];
    pub(super) static LENGTH: U32 = 0u;
    pub(super) static FIRST_ITEM: U32 = 0u;
    pub(super) static SECOND_ITEM: U32 = 0u;
    pub(super) static OUT_OF_BOUND_ITEM: U32 = 0u;
    pub(super) static NESTED_ARRAY_ITEM: U32 = 0u;
    pub(super) static ITER_SUM: U32 = 0u;
    pub(super) static ENUMERATED_ITER_SUM: U32 = 0u;

    #[compute]
    fn run() {
        *LENGTH = FROM_ITEMS.len();
        *FIRST_ITEM = FROM_ITEMS[0u];
        *SECOND_ITEM = FROM_ITEMS[1u];
        *OUT_OF_BOUND_ITEM = FROM_ITEMS[4u];
        *NESTED_ARRAY_ITEM = NESTED[1u][0u];
        NESTED[2u][0u] = 9u;
        NESTED[1u] = [10u, 11u];
        DEEP[0u][0u][0u][0u][0u] = 10u;
        for inner_array in *NESTED {
            for inner_value in *inner_array {
                *ITER_SUM += *inner_value;
            }
        }
        for inner_array in *NESTED {
            for (index, inner_value) in *inner_array {
                *ENUMERATED_ITER_SUM += *inner_value + index;
            }
        }
    }
}
