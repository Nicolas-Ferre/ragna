use ragna::{assert_same_texture, App};

#[test]
pub fn read_not_rendered_target() {
    let app = App::default().texture();
    assert_same_texture("target#default_not_rendered", &app.read_target());
}

#[test]
pub fn read_rendered_target() {
    let app = App::default().texture().run(1);
    assert_same_texture("target#default_rendered", &app.read_target());
}
