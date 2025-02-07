#![allow(missing_docs, clippy::unwrap_used)]

use itertools::Itertools;
use std::path::Path;
use std::process::Command;
use std::{env, fs};

#[test]
pub fn run_compile_tests() {
    let output = Command::new("cargo")
        .current_dir("../../compile_tests")
        .arg("check")
        .env_remove("CARGO_TERM_COLOR")
        .env_remove("RUSTC_BOOTSTRAP")
        .output()
        .unwrap();
    let grouped_errors = String::from_utf8(output.stderr)
        .unwrap()
        .lines()
        .skip_while(|line| !line.contains("ragna_compile_tests"))
        .skip(1)
        .take_while(|line| {
            !line.contains("Some errors have detailed explanations")
                && !line.contains("could not compile")
        })
        .join("\n")
        .split("\nerror")
        .enumerate()
        .map(|(index, error)| format!("{}{}", if index == 0 { "" } else { "error" }, error))
        .into_group_map_by(|error| error_path(error));
    let mut is_new = false;
    for (path, errors) in grouped_errors {
        let errors = errors.join("\n").replace(
            &*Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .unwrap()
                .parent()
                .unwrap()
                .to_string_lossy(),
            ".",
        );
        if fs::exists(&path).unwrap() {
            let expected = String::from_utf8(fs::read(&path).unwrap()).unwrap();
            assert_eq!(expected, errors, "invalid errors for {path}");
        } else {
            fs::write(&path, errors).unwrap();
            is_new = true;
        }
    }
    assert!(
        !is_new,
        "expected errors saved on disk, please check and rerun the tests"
    );
}

fn error_path(error: &str) -> String {
    let name_prefix = "--> src/";
    let name_start_pos = error.find(name_prefix).unwrap() + name_prefix.len();
    let name_end_pos = error[name_start_pos..].find(".rs").unwrap() + name_start_pos;
    format!(
        "../../compile_tests/expected/{}.stderr",
        &error[name_start_pos..name_end_pos]
    )
}
