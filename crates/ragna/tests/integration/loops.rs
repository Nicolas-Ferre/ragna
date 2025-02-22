use ragna::App;

#[test]
pub fn run_loops() {
    let app = App::default().with_module(gpu::register).run(1);
    assert_eq!(app.read(gpu::WHILE_RESULT), Some(45));
}

#[ragna::gpu]
mod gpu {
    use ragna::I32;

    pub(super) static WHILE_RESULT: I32 = 0;

    #[compute]
    fn run_if() {
        let i = 0;
        while i < 10 {
            WHILE_RESULT += i;
            i += 1;
        }
    }
}
