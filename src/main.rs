use anyhow::{bail, Context, Result};
use cargo_toml::Value;
use fs_extra::dir::CopyOptions;
use std::{io::Write, os::unix::prelude::OpenOptionsExt, process::Command};

const APPDIRPATH: &str = "target/cargo-appimage.AppDir/";

fn main() -> Result<()> {
    let appdirpath = std::path::Path::new(APPDIRPATH);
    match Command::new("cargo")
        .arg("build")
        .arg("--release")
        .args(std::env::args().nth(2))
        .status()
    {
        Ok(_) => {}
        Err(_) => bail!("Failed to build package"),
    }

    if !std::path::Path::new("./icon.png").exists() {
        std::fs::write("./icon.png", &[]).context("Failed to generate icon.png")?;
    }

    let meta = cargo_toml::Manifest::<Value>::from_path_with_metadata("./Cargo.toml")
        .context("Cannot find Cargo.toml")?
        .package
        .context("Cannot load metadata from Cargo.toml")?;

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

    fs_extra::dir::create_all(appdirpath.join("usr/bin"), false)?;
    std::fs::copy(
        format!("target/release/{}", meta.name),
        appdirpath.join("usr/bin/bin"),
    )
    .context("Cannot find binary file")?;
    std::fs::copy("./icon.png", appdirpath.join("icon.png")).context("Cannot find icon.png")?;
    fs_extra::copy_items(
        &assets,
        appdirpath,
        &CopyOptions {
            overwrite: true,
            buffer_size: 0,
            copy_inside: true,
            ..Default::default()
        },
    )
    .context("Error copying assets")?;
    std::fs::write(
        appdirpath.join("cargo-appimage.desktop"),
        format!(
            "[Desktop Entry]\nName={}\nExec=bin\nIcon=icon\nType=Application\nCategories=Utility;",
            meta.name
        ),
    )?;
    std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .mode(0o770)
        .open(appdirpath.join("AppRun"))?.write("#!/bin/sh\n\nHERE=\"$(dirname \"$(readlink -f \"${0}\")\")\"\nEXEC=\"${HERE}/usr/bin/bin\"\nexec \"${EXEC}\"".as_bytes())?;
    Command::new("appimagetool")
        .arg(appdirpath)
        .env("ARCH", platforms::target::TARGET_ARCH.as_str())
        .env("VERSION", meta.version)
        .status()?;
    Ok(())
}
