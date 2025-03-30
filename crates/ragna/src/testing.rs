use crate::TextureData;
use image::ColorType;
use std::path::PathBuf;
use std::{env, fs};

/// Asserts a texture is the same as the expected texture.
///
/// If the expected texture is not yet generated, it is saved in
/// `$CARGO_MANIFEST_DIR/tests/expected/{key}.png` and the function panics. At the next function
/// run, the function shouldn't panic if actual texture have not changed.
///
/// If there is a difference between expected and actual texture, a diff texture is saved in a
/// temporary folder and the function panics with a message containing the path to the diff
/// texture.
///
/// The generated diff texture is a black texture, with white color for pixels that are
/// different.
///
/// # Panics
///
/// This will panic if:
/// - the expected and actual textures are different.
/// - the actual texture doesn't match the
///   expected one saved in `$CARGO_MANIFEST_DIR/tests/expected/{key}.png`.
/// - there is an I/O error while reading or writing the expected texture or the diff texture.
///
/// # Examples
///
/// ```rust
/// # use ragna::{assert_same_texture, App};
/// #
/// # fn no_run() {
/// let app = App::default().texture((800, 600)).run(1);
/// assert_same_texture("expected_filename", &app.read_target());
/// # }
/// ```
pub fn assert_same_texture(key: &str, actual: &TextureData) {
    let expected_folder = env::var("CARGO_MANIFEST_DIR")
        .expect("`CARGO_MANIFEST_DIR` environment variable not set")
        + "/tests/expected";
    let expected_file: PathBuf = format!("{expected_folder}/{key}.png").into();
    if expected_file.exists() {
        let image = image::open(&expected_file).expect("cannot read expected texture from disk");
        assert_eq!(image.width(), actual.size.0, "texture width is different");
        assert_eq!(image.height(), actual.size.1, "texture height is different");
        let expected_data = image.to_rgba8().into_raw();
        if expected_data != actual.buffer {
            let diff_data = texture_diff(&actual.buffer, &expected_data);
            let diff_file = env::temp_dir().join(format!("diff_{key}.png"));
            image::save_buffer(
                &diff_file,
                &diff_data,
                actual.size.0,
                actual.size.1,
                ColorType::Rgba8,
            )
            .expect("cannot save texture diff");
            panic!("texture is different (diff saved in {diff_file:?})")
        }
    } else {
        fs::create_dir_all(expected_folder).expect("cannot create folder for expected texture");
        image::save_buffer(
            &expected_file,
            &actual.buffer,
            actual.size.0,
            actual.size.1,
            ColorType::Rgba8,
        )
        .expect("cannot save expected texture");
        panic!("expected texture saved, need to rerun the test");
    }
}

fn texture_diff(texture1: &[u8], texture2: &[u8]) -> Vec<u8> {
    texture1
        .chunks(4)
        .zip(texture2.chunks(4))
        .flat_map(|(e, a)| {
            if a == e {
                [0, 0, 0, 255]
            } else {
                [255, 255, 255, 255]
            }
        })
        .collect()
}
