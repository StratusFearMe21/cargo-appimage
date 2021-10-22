use std::process::Command;

use anyhow::Context;

fn main() -> anyhow::Result<()> {
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
  let meta = cargo_toml::Manifest::from_path("./Cargo.toml")
    .context("Cannot find Cargo.toml")?
    .package
    .unwrap();
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
    Ok(())
}
