use ragna::{App, Cpu};

#[test]
#[should_panic = "variable should be global to be registered"]
pub fn run_app_multiple_times() {
    let glob = ragna::create_glob("", 0, || (0..1).to_gpu());
    App::default().with_glob(glob.start);
}
