use ragna::{assert_same_texture, App};

#[test]
pub fn read_not_rendered_target() {
    let app = App::default().texture((4, 4));
    assert_same_texture("target#default_not_rendered", &app.read_target());
}

#[test]
pub fn read_rendered_target() {
    let app = App::default().texture((4, 4)).run(1);
    assert_same_texture("target#default_rendered", &app.read_target());
}

#[test]
pub fn configure_background_color() {
    let app = App::default()
        .texture((4, 4))
        .with_background_color((0., 0., 1., 1.))
        .run(1);
    assert_same_texture("target#blue_background", &app.read_target());
}
