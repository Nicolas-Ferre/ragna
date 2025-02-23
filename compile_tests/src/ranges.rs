fn main() {}

#[ragna::gpu]
mod gpu {
    use ragna::{Range, I32};

    static RANGE_WITHOUT_START: Range<I32> = ..5;
    static RANGE_WITHOUT_END: Range<I32> = 3..;
    static UNSUPPORTED_RANGE_FORM: Range<I32> = 3..=5;
}
