#![allow(missing_docs, clippy::unwrap_used)]

use itertools::Itertools;
use std::path::Path;
use std::process::Command;
use std::{env, fs};

const LIB_RS_PATH: &str = "../../compile_tests/src/lib.rs";
const EXPECTED_PATH: &str = "../../compile_tests/expected";

#[test]
pub fn run_compile_tests() {
    uncomment_compile_tests();
    let output = Command::new("cargo")
        .arg("check")
        .arg("--package=ragna_compile_tests")
        .env_remove("CARGO_TERM_COLOR")
        .env_remove("RUSTC_BOOTSTRAP")
        .output()
        .unwrap();
    comment_compile_tests();
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

fn uncomment_compile_tests() {
    let uncommented = fs::read_to_string(LIB_RS_PATH)
        .unwrap()
        .lines()
        .map(|l| l.replacen("//", "", 1))
        .join("\n");
    fs::write(LIB_RS_PATH, uncommented).unwrap();
}

fn comment_compile_tests() {
    let uncommented = fs::read_to_string(LIB_RS_PATH)
        .unwrap()
        .lines()
        .map(|l| format!("//{l}"))
        .join("\n");
    fs::write(LIB_RS_PATH, uncommented).unwrap();
}

fn error_path(error: &str) -> String {
    let name_prefix = "--> compile_tests/src/";
    let name_start_pos = error.find(name_prefix).unwrap() + name_prefix.len();
    let name_end_pos = error[name_start_pos..].find(".rs").unwrap() + name_start_pos;
    format!(
        "{EXPECTED_PATH}/{}.stderr",
        &error[name_start_pos..name_end_pos]
    )
}
