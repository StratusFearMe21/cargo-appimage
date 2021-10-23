use anyhow::{Context, Result};
use cargo_toml::Value;
use fs_extra::copy_items;
use fs_extra::dir::CopyOptions;
use std::process::Command;

fn main() -> Result<()> {
    match Command::new("cargo").arg("build").arg("--release").status() {
        Ok(_) => {}
        Err(_) => panic!("Failed to build package"),
    }
    match std::path::Path::new("./icon.png").exists() {
        true => {}
        false => {
            Command::new("touch")
                .arg("icon.png")
                .status()
                .context("Failed to generate icon.png")?;
        }
    }

    let meta = cargo_toml::Manifest::<Value>::from_path_with_metadata("./Cargo.toml")
        .context("Cannot find Cargo.toml")?
        .package
        .unwrap();

    let assets = match &meta.metadata.unwrap_or_else(|| Value::Array(vec![])) {
        Value::Table(t) => match t.get("appimage") {
            Some(Value::Table(t)) => match t.get("assets") {
                Some(Value::Array(v)) => v
                    .to_vec()
                    .into_iter()
                    .filter_map(|v| match v {
                        Value::String(s) => Some(s),
                        _ => None,
                    })
                    .collect(),
                _ => vec![],
            },
            _ => vec![],
        },
        _ => vec![],
    };

    std::fs::create_dir("target/cargo-appimage.AppDir").unwrap_or(());
    std::fs::create_dir("target/cargo-appimage.AppDir/usr").unwrap_or(());
    std::fs::create_dir("target/cargo-appimage.AppDir/usr/bin").unwrap_or(());
    std::fs::copy(
        format!("target/release/{}", meta.name),
        "target/cargo-appimage.AppDir/usr/bin/bin",
    )
    .context("Cannot find binary file")?;
    std::fs::copy("./icon.png", "target/cargo-appimage.AppDir/icon.png")
        .context("Cannot find icon.png")?;
    copy_items(
        &assets,
        "target/cargo-appimage.AppDir/",
        &CopyOptions {
            overwrite: true,
            skip_exist: false,
            buffer_size: 0,
            copy_inside: true,
            content_only: false,
            depth: 0,
        },
    )
    .context("Error copying assets")?;
    std::fs::write(
        "target/cargo-appimage.AppDir/cargo-appimage.desktop",
        format!(
            "[Desktop Entry]\nName={}\nExec=bin\nIcon=icon\nType=Application\nCategories=Utility;",
            meta.name
        ),
    )
    .unwrap_or(());
    std::fs::write(
        "target/cargo-appimage.AppDir/AppRun",
        "#!/bin/sh\n\nHERE=\"$(dirname \"$(readlink -f \"${0}\")\")\"\nEXEC=\"${HERE}/usr/bin/bin\"\nexec \"${EXEC}\"",
        )
        .unwrap_or(());
    Command::new("chmod")
        .arg("+x")
        .arg("target/cargo-appimage.AppDir/AppRun")
        .status()?;
    Command::new("appimagetool")
        .arg("target/cargo-appimage.AppDir/")
        .env("ARCH", platforms::target::TARGET_ARCH.as_str())
        .status()?;
    std::fs::write(
        "target/cargo-appimage.AppDir/AppRun",
        "#!/bin/sh\n\nHERE=\"$(dirname \"$(readlink -f \"${0}\")\")\"\nEXEC=\"${HERE}/usr/bin/bin\"\nexec \"${EXEC}\"",
        )
        .unwrap_or(());
    Command::new("chmod")
        .arg("+x")
        .arg("target/cargo-appimage.AppDir/AppRun")
        .status()?;
    Command::new("appimagetool")
        .arg("target/cargo-appimage.AppDir/")
        .env("ARCH", platforms::target::TARGET_ARCH.as_str())
        .env("VERSION", meta.version)
        .status()?;
    Ok(())
}
