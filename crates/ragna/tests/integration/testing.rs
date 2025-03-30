use image::ImageError;
use ragna::{assert_same_texture, App};
use std::panic::AssertUnwindSafe;
use std::path::Path;
use std::{fs, panic};

#[test]
pub fn compare_to_not_existing_expected_texture() {
    let app = App::default().texture((4, 4)).run(1);
    let result = panic::catch_unwind(AssertUnwindSafe(|| {
        assert_same_texture("testing#non_existing", &app.read_target());
    }));
    let actual_path = "tests/expected/testing#non_existing.png";
    let actual_image = load_image_data(actual_path);
    let expected_image = load_image_data("tests/expected/target#default_rendered.png");
    assert!(fs::remove_file(actual_path).is_ok());
    assert!(result.is_err());
    assert_eq!(actual_image.ok(), expected_image.ok());
}

#[test]
fn compare_to_same_texture() {
    let app = App::default().texture((4, 4)).run(1);
    assert_same_texture("target#default_rendered", &app.read_target());
}

#[test]
#[should_panic = "texture is different"]
fn compare_to_different_texture() {
    let app = App::default().texture((4, 4)).run(1);
    assert_same_texture("testing#different_pixels", &app.read_target());
}

#[test]
fn generate_diff_texture() {
    let app = App::default().texture((4, 4)).run(1);
    let result = panic::catch_unwind(AssertUnwindSafe(|| {
        assert_same_texture("testing#different_pixels", &app.read_target());
    }));
    assert!(result.is_err());
    let actual_diff =
        load_image_data(std::env::temp_dir().join("diff_testing#different_pixels.png"));
    let expected_diff = load_image_data("tests/expected/testing#different_pixels_diff.png");
    assert_eq!(actual_diff.ok(), expected_diff.ok());
}

#[test]
#[should_panic = "texture width is different"]
fn compare_to_texture_with_different_width() {
    let app = App::default().texture((4, 4)).run(1);
    assert_same_texture("testing#different_width", &app.read_target());
}

#[test]
#[should_panic = "texture height is different"]
fn compare_to_texture_with_different_height() {
    let app = App::default().texture((4, 4)).run(1);
    assert_same_texture("testing#different_height", &app.read_target());
}

fn load_image_data(path: impl AsRef<Path>) -> Result<Vec<u8>, ImageError> {
    Ok(image::open(path)?.to_rgba8().into_raw())
}
