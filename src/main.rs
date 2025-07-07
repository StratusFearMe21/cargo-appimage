use anyhow::{Context, Result, bail};
use cargo_toml::Value;
use fs_extra::dir::CopyOptions;
use std::{
    io::{Read, Write},
    process::Stdio,
};

// TODO use clap for arg management instead of raw handle - 20/3/2025
fn main() -> Result<()> {
    // Create and execute cargo build command.
    let mut command = std::process::Command::new("cargo");
    command.arg("build");
    if !std::env::args()
        .skip(2)
        .any(|arg| arg.starts_with("--profile="))
    {
        command.arg("--release");
    }
    command.args(std::env::args().skip(2));
    command.status().context("Failed to build package")?;

    let cargo_metadata = cargo_metadata::MetadataCommand::new()
        .exec()
        .context("Failed to execute cargo metadata")?;
    let target_prefix = cargo_metadata.target_directory;

    while !std::path::Path::new("Cargo.toml").exists() {
        if std::env::current_dir()? == std::path::Path::new("/") {
            bail!("No Cargo.toml found in any parent dirs");
        }
        std::env::set_current_dir("..").context("Cannot chdir into previous directory")?;
    }

    let file = std::fs::File::open("Cargo.toml").context("Failed to open Cargo.toml")?;
    let mmap = unsafe { memmap::Mmap::map(&file)? };
    let mut meta = cargo_toml::Manifest::<Value>::from_slice(mmap.as_ref())
        .context("Failed to parse Cargo.toml")?;

    meta.complete_from_path_and_workspace::<cargo_toml::Value>(
        std::path::Path::new("./Cargo.toml"),
        None,
    )
    .context("Could not fill in the gaps in Cargo.toml")?;

    let pkg = meta
        .package
        .context("Cannot load metadata from Cargo.toml")?;
    let assets;
    let mut icon_path = "./icon.png";
    let mut desktop_entry_path = "./cargo-appimage.desktop";

    let target = {
        let profile = std::env::args()
            .skip(2)
            .find(|arg| arg.starts_with("--profile="))
            .map(|arg| arg.split_at(10).1.to_string())
            .unwrap_or_else(|| "release".into());
        std::env::args()
            .skip(2)
            .find(|arg| arg.starts_with("--target="))
            .map(|arg| format!("{}/{}", arg.split_at(9).1, profile))
            .unwrap_or_else(|| profile)
    };
    let link_deps;
    let mut link_exclude_list = Vec::with_capacity(0);
    let mut args = vec![];

    if let Some(meta) = pkg.metadata.as_ref() {
        match meta {
            Value::Table(t) => match t.get("appimage") {
                Some(Value::Table(t)) => {
                    match t.get("assets") {
                        Some(Value::Array(v)) => {
                            assets = v
                                .iter()
                                .filter_map(|v| match v {
                                    Value::String(s) => Some(s),
                                    _ => None,
                                })
                                .collect()
                        }
                        _ => assets = Vec::with_capacity(0),
                    }
                    match t.get("icon") {
                        Some(Value::String(s)) => {
                            icon_path = s;
                        }
                        _ => {},
                    }
                    match t.get("desktop_entry") {
                        Some(Value::String(s)) => {
                            desktop_entry_path = s;
                        }
                        _ => {},
                    }
                    match t.get("auto_link") {
                        Some(Value::Boolean(v)) => link_deps = v.to_owned(),
                        _ => link_deps = false,
                    }
                    if let Some(Value::Array(v)) = t.get("args") {
                        args = v
                            .iter()
                            .filter_map(|v| match v {
                                Value::String(s) => Some(s),
                                _ => None,
                            })
                            .collect()
                    }
                    if let Some(Value::Array(arr)) = t.get("auto_link_exclude_list") {
                        for v in arr.iter() {
                            if let Value::String(s) = v {
                                link_exclude_list.push(glob::Pattern::new(s).context(
                                    "Auto-link exclude list item not a valid glob pattern",
                                )?);
                            }
                        }
                    }
                }
                _ => {
                    assets = Vec::with_capacity(0);
                    link_deps = false
                }
            },
            _ => {
                assets = Vec::with_capacity(0);
                link_deps = false
            }
        };
    } else {
        assets = Vec::with_capacity(0);
        link_deps = false;
    }

    // Prepare the AppDir directory structure
    for current_product in meta.bin {
        let name = current_product.name.unwrap_or(pkg.name.clone());
        let appdirpath = std::path::Path::new(&target_prefix).join(name.clone() + ".AppDir");
        fs_extra::dir::create_all(appdirpath.join("usr"), true)
            .with_context(|| format!("Error creating {}", appdirpath.join("usr").display()))?;

        fs_extra::dir::create_all(appdirpath.join("usr/bin"), true)
            .with_context(|| format!("Error creating {}", appdirpath.join("usr/bin").display()))?;
        if link_deps {
            if !std::path::Path::new("libs").exists() {
                std::fs::create_dir("libs").context("Could not create libs directory")?;
            }
            let awk = std::process::Command::new("awk")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .arg("NF == 4 {print $3}; NF == 2 {print $1}")
                .spawn()
                .context("Could not start awk")?;

            awk.stdin
                .context("Make sure you have awk on your system")?
                .write_all(
                    &std::process::Command::new("ldd")
                        .arg(format!("{}/{}/{}", target_prefix, &target, &name))
                        .output()
                        .with_context(|| {
                            format!(
                                "Failed to run ldd on {}/{}/{}",
                                target_prefix, &target, &name
                            )
                        })?
                        .stdout,
                )?;

            let mut linkedlibs = String::new();
            awk.stdout
                .context("Unknown error ocurred while running awk")?
                .read_to_string(&mut linkedlibs)?;

            fs_extra::dir::create("libs", true).context("Failed to create libs dir")?;

            for line in linkedlibs.lines() {
                if line.starts_with('/') && !std::path::Path::new("libs").join(&line[1..]).exists()
                {
                    std::os::unix::fs::symlink(
                        line,
                        std::path::Path::new("libs").join(
                            std::path::Path::new(line)
                                .file_name()
                                .with_context(|| format!("No filename for {}", line))?,
                        ),
                    )
                    .with_context(|| {
                        format!(
                            "Error symlinking {} to {}",
                            line,
                            std::path::Path::new("libs").join(&line[1..]).display()
                        )
                    })?;
                }
            }
        }

        if std::path::Path::new("libs").exists() {
            for i in std::fs::read_dir("libs").context("Could not read libs dir")? {
                let path = &i?.path();

                // Skip if it matches the exclude list.
                if let Some(file_name) = path.file_name().and_then(|p| p.to_str()) {
                    if link_exclude_list.iter().any(|p| p.matches(file_name)) {
                        continue;
                    }
                }

                let link = std::fs::read_link(path)
                    .with_context(|| format!("Error reading link in libs {}", path.display()))?;

                fs_extra::dir::create_all(
                    appdirpath.join(
                        &link
                            .parent()
                            .with_context(|| format!("Lib {} has no parent dir", &link.display()))?
                            .to_str()
                            .with_context(|| format!("{} is not valid Unicode", link.display()))?
                            [1..],
                    ),
                    false,
                )?;
                let dest = appdirpath.join(
                    &link
                        .to_str()
                        .with_context(|| format!("{} is not valid Unicode", link.display()))?[1..],
                );
                std::fs::copy(&link, &dest).with_context(|| {
                    format!("Error copying {} to {}", &link.display(), dest.display())
                })?;
            }
        }

        // Copy app executable
        std::fs::copy(
            format!("{}/{}/{}", target_prefix, &target, &name),
            appdirpath.join(format!("usr/bin/{}", &name)),
        )
        .with_context(|| {
            format!(
                "Cannot find binary file at {}/{}/{}",
                target_prefix, &target, &name
            )
        })?;

        // Copy icon
        {
            let icon_path_object = std::path::Path::new(icon_path);
            if !icon_path_object.exists() {
                println!(
                    "[CARGO APPIMAGE] {icon_path} does not exist. No icon will be used for your AppImage."
                );
            } else {
                // Assuming all image files have an extension
                let extension = icon_path_object
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .context("No icon extension found")?;

                let destination = appdirpath.join(format!("{name}.{extension}"));
                std::fs::copy(icon_path_object, destination)
                    .context(format!("Cannot copy {icon_path}"))?;
            }
        }

        // Copy desktop entry
        if !std::path::Path::new(desktop_entry_path).exists() {
            std::fs::write(
                appdirpath.join("cargo-appimage.desktop"),
                format!(
                    "[Desktop Entry]\nName={}\nExec={}\nIcon={}\nType=Application\nCategories=Utility;", name
                    , name, name),
            )
                .with_context(|| {
                    format!(
                        "Error writing desktop file {}",
                        appdirpath.join("cargo-appimage.desktop").display()
                    )
                })?;
        } else {
            std::fs::copy(desktop_entry_path, appdirpath.join("cargo-appimage.desktop"))
                .context(format!("Cannot find {desktop_entry_path}"))?;
        }

        // Copy assets
        fs_extra::copy_items(
            &assets,
            appdirpath.as_path(),
            &CopyOptions {
                overwrite: true,
                buffer_size: 0,
                copy_inside: true,
                ..Default::default()
            },
        )
        .context("Error copying assets")?;

        // Copy runner
        std::fs::copy(
            std::path::PathBuf::from(std::env::var("HOME")?)
                .join(std::env::var("CARGO_HOME").unwrap_or_else(|_| ".cargo".to_string()))
                .join("bin/cargo-appimage-runner"),
            appdirpath.join("AppRun"),
        )
        .with_context(|| {
            format!(
                "Error copying {} to {}",
                std::path::PathBuf::from(std::env::var("HOME").unwrap())
                    .join(std::env::var("CARGO_HOME").unwrap_or_else(|_| ".cargo".to_string()))
                    .join("bin/cargo-appimage-runner")
                    .display(),
                appdirpath.join("AppRun").display()
            )
        })?;

        // Launch appimagetool
        let mut bin_args = args.to_vec();
        let appdirpath = appdirpath.into_os_string().into_string().unwrap();
        bin_args.push(&appdirpath);

        std::fs::create_dir_all(format!("{}/appimage", &target_prefix))
            .context("Unable to create output dir")?;
        std::process::Command::new("appimagetool")
            .args(bin_args)
            .arg(format!("{}/appimage/{}.AppImage", &target_prefix, &name))
            .env("ARCH", std::env::consts::ARCH)
            .env("VERSION", pkg.version())
            .status()
            .context("Error occurred: make sure that appimagetool is installed")?;
    }

    Ok(())
}
