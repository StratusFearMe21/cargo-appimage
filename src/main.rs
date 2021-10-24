use anyhow::{bail, Context, Result};
use cargo_toml::Value;
use fs_extra::dir::CopyOptions;
use std::{
    io::{Read, Write},
    process::{Command, Stdio},
};

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
    let assets;
    let link_deps;

    match meta
        .metadata
        .unwrap_or_else(|| Value::Array(Vec::with_capacity(0)))
    {
        Value::Table(t) => match t.get("appimage") {
            Some(Value::Table(t)) => {
                match t.get("assets") {
                    Some(Value::Array(v)) => {
                        assets = v
                            .to_vec()
                            .into_iter()
                            .filter_map(|v| match v {
                                Value::String(s) => Some(s),
                                _ => None,
                            })
                            .collect()
                    }
                    _ => assets = Vec::with_capacity(0),
                }
                match t.get("link_deps") {
                    Some(Value::Boolean(v)) => link_deps = v.to_owned(),
                    _ => link_deps = true,
                }
            }
            _ => {
                assets = Vec::with_capacity(0);
                link_deps = true
            }
        },
        _ => {
            assets = Vec::with_capacity(0);
            link_deps = true
        }
    };

    fs_extra::dir::create_all(appdirpath.join("usr/bin"), false)?;
    if link_deps {
        let awk = std::process::Command::new("awk")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .arg("NF == 4 {print $3}; NF == 2 {print $1}")
            .spawn()?;

        awk.stdin
            .context("Make sure you have awk on your system")?
            .write_all(
                &std::process::Command::new("ldd")
                    .arg(format!("target/release/{}", meta.name))
                    .output()?
                    .stdout,
            )?;

        let mut linkedlibs = String::new();
        awk.stdout
            .context("Unknown error ocurred while running awk")?
            .read_to_string(&mut linkedlibs)?;

        for line in linkedlibs.lines() {
            if line.starts_with("/") {
                fs_extra::dir::create_all(
                    appdirpath.join(
                        std::path::Path::new(&line[1..])
                            .parent()
                            .context("Lib has no parent path")?,
                    ),
                    false,
                )?;
                std::fs::copy(line, appdirpath.join(&line[1..]))?;
            }
        }
    }
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
    std::fs::copy(
        std::path::PathBuf::from(std::env::var("HOME")?).join(".cargo/bin/cargo-appimage-runner"),
        appdirpath.join("AppRun"),
    )?;

    Command::new("appimagetool")
        .arg(appdirpath)
        .env("ARCH", platforms::target::TARGET_ARCH.as_str())
        .env("VERSION", meta.version)
        .status()
        .context("Error occurred: make sure that appimagetool is installed")?;
    Ok(())
}
