#![cfg(test)]

use core::str::from_utf8;
use std::{
    env::{self, temp_dir},
    fs::{create_dir_all, write},
    process::Command,
};

#[test]
fn e2e() {
    let e2e_dir = temp_dir().join("unfmt_e2e");
    create_dir_all(e2e_dir.join("src")).expect("failed to create temp dir");

    write(
        e2e_dir.join("src/main.rs"),
        r#"use unfmt::unformat;fn main() {unformat!("hello {}", "hello world");}"#,
    )
    .expect("failed to write file");

    let mut cargo_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    if cfg!(windows) {
        cargo_dir = cargo_dir.replace('\\', "/");
    }
    write(
        e2e_dir.join("Cargo.toml"),
        format!(
            r#"
[package]
name = "unfmt_e2e"
version = "0.1.0"
edition = "2021"

[dependencies.unfmt]
path = "{cargo_dir}"
"#
        ),
    )
    .expect("failed to write file");

    let output = Command::new("cargo")
        .arg("run")
        .current_dir(&e2e_dir)
        .output()
        .expect("failed to run cargo");

    assert!(
        output.status.success(),
        "cargo run failed in {}\nstderr: {}\nstdout: {}",
        e2e_dir.display(),
        from_utf8(&output.stderr).expect("failed to convert stderr to string"),
        from_utf8(&output.stdout).expect("failed to convert stdout to string")
    );
}
