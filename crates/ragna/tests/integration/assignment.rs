use ragna::App;

#[test]
pub fn assign_values() {
    let app = App::default().with_module(gpu::register).run(1);
    assert_eq!(app.read(gpu::FROM_VAR), Some(10));
    assert_eq!(app.read(gpu::FROM_MODIFIED_VAR), Some(20));
    assert_eq!(app.read(gpu::FROM_CONSTANT), Some(30));
    assert_eq!(app.read(gpu::FROM_GLOB), Some(30));
}

#[ragna::gpu]
mod gpu {
    use ragna::I32;

    const CONSTANT: I32 = 30;

    pub(super) static FROM_VAR: I32 = 0;
    pub(super) static FROM_MODIFIED_VAR: I32 = 0;
    pub(super) static FROM_CONSTANT: I32 = 0;
    pub(super) static FROM_GLOB: I32 = 0;

    #[compute]
    fn run() {
        let var: I32 = 10;
        FROM_VAR = var;
        let modified_var = 10;
        modified_var = 20;
        FROM_MODIFIED_VAR = modified_var;
        FROM_CONSTANT = CONSTANT;
        FROM_GLOB = FROM_CONSTANT;
    }
}
