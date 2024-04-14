use std::{
    env::{self, temp_dir},
    fs::{create_dir_all, write},
};

#[test]
fn test_e2e() {
    let e2e_dir = temp_dir().join("unfmt_e2e");
    create_dir_all(e2e_dir.join("src")).expect("failed to create temp dir");

    write(
        e2e_dir.join("src/main.rs"),
        r#"use unfmt::unformat;fn main() {unformat!("hello {}", "hello world");}"#,
    )
    .expect("failed to write file");

    let mut cargo_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    if cfg!(windows) {
        cargo_dir = cargo_dir.replace("\\", "/");
    }
    write(
        e2e_dir.join("Cargo.toml"),
        format! { r#"
[package]
name = "unfmt_e2e"
version = "0.1.0"
edition = "2021"

[dependencies.unfmt]
path = "{cargo_dir}"
"#},
    )
    .expect("failed to write file");

    let output = std::process::Command::new("cargo")
        .arg("run")
        .current_dir(&e2e_dir)
        .output()
        .expect("failed to run cargo");

    if !output.status.success() {
        println!("tempdir: {}", e2e_dir.display());
        println!(
            "stderr: {}",
            std::str::from_utf8(&output.stderr).expect("failed to convert stdout to string")
        );
        println!(
            "stdout: {}",
            std::str::from_utf8(&output.stdout).expect("failed to convert stdout to string")
        );
    }

    assert!(output.status.success());
}
