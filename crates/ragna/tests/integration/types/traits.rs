use ragna::App;

#[test]
pub fn use_structs() {
    let app = App::default().with_module(gpu::register).testing().run(1);
    assert_eq!(app.read(*gpu::ADD_RESULT).unwrap().inner, 7);
}

#[ragna::gpu]
mod gpu {
    use ragna::{Gpu, I32};
    use std::ops::Add;

    pub(super) struct Wrapped<T: Gpu> {
        pub(super) inner: T,
    }

    impl<T> Add for Wrapped<T>
    where
        T: Gpu + Add<T, Output = T>,
    {
        type Output = Self;

        fn add(self, rhs: Self) -> Self::Output {
            Self {
                inner: self.inner + rhs.inner,
            }
        }
    }

    pub(super) static ADD_RESULT: Wrapped<I32> = Wrapped::<I32> { inner: 0 };
    pub(super) static TRAIT_VALUE_RESULT: I32 = 0;
    pub(super) static TRAIT_DEFAULT_IMPL_RESULT: I32 = 0;

    trait TestTrait {
        type ReturnedValue;

        fn returned_value(&self) -> Self::ReturnedValue;

        fn default_impl(copied: I32) -> I32 {
            copied += 1; // shouldn't have impact outside the function
            42
        }
    }

    impl<T: Gpu> TestTrait for Wrapped<T> {
        type ReturnedValue = T;

        fn returned_value(&self) -> Self::ReturnedValue {
            self.inner
        }
    }

    #[compute]
    fn run() {
        *ADD_RESULT = Wrapped::<I32> { inner: 2 } + Wrapped::<I32> { inner: 5 };
        *TRAIT_VALUE_RESULT = ADD_RESULT.returned_value();
        *TRAIT_DEFAULT_IMPL_RESULT = Wrapped::<I32>::default_impl(ADD_RESULT.inner);
    }
}
