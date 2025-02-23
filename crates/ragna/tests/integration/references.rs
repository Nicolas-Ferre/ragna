use ragna::App;

#[test]
pub fn use_references() {
    let app = App::default().with_module(gpu::register).run(1);
    assert_eq!(app.read(*gpu::RESULT), Some(26));
}

#[ragna::gpu]
mod gpu {
    use ragna::I32;

    pub(super) static RESULT: I32 = 5;

    #[compute]
    fn run() {
        increment(&RESULT);
        let reference = &RESULT;
        add(add(reference, 10), 10);
        let value = value();
        value += 1;
    }

    fn increment(value: &I32) {
        *value += 1;
    }

    fn add(value: &I32, added: I32) -> &I32 {
        *value += added;
        value
    }

    fn value() -> I32 {
        *RESULT
    }
}
