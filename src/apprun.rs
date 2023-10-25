use anyhow::Context;
use std::fs;

fn main() -> anyhow::Result<()> {
    let here_dir = std::env::current_exe()?;
    let parent = here_dir
        .parent()
        .with_context(|| format!("{} has no parent directory", &here_dir.display()))?;
    std::env::set_current_dir(parent)?;
    std::env::set_var(
        "LD_LIBRARY_PATH",
        format!("{}/usr/lib/:{}/usr/lib/i386-linux-gnu/:{}/usr/lib/x86_64-linux-gnu/:{}/usr/lib32/:{}/usr/lib64/:{}/lib/:{}/lib/i386-linux-gnu/:{}/lib/x86_64-linux-gnu/:{}/lib32/:{}/lib64/{}", parent.display(), parent.display(), parent.display(), parent.display(), parent.display(), parent.display(), parent.display(), parent.display(), parent.display(), parent.display(), if let Ok(ldlibpath) = std::env::var("LD_LIBRARY_PATH") { ":".to_string() + &ldlibpath } else { String::new() }),
    );
    std::env::set_var(
        "PATH",
        format!(
            "{}/usr/bin:{}/bin{}",
            parent.display(),
            parent.display(),
            if let Ok(path) = std::env::var("PATH") {
                ":".to_string() + &path
            } else {
                String::new()
            }
        ),
    );
    std::env::set_var(
        "XDG_DATA_DIRS",
        format!(
            "XDG_DATA_DIRS={}:{}",
            parent.join("usr/share").display(),
            std::env::var("XDG_DATA_DIRS").unwrap_or(String::new())
        ),
    );

    let Some(executable) = fs::read_dir(parent.join("usr/bin/"))?.next() else {
        eprintln!("Error: Executable file not found");
        return Ok(());
    };

    let file_name = executable?.file_name();

    let Some(executable_name) = file_name.to_str() else {
        eprintln!("Error: Failed to get executable name");
        return Ok(());
    };

    let err = exec::execvp(
        parent.join(format!("usr/bin/{executable_name}")),
        std::env::args(),
    );
    eprintln!("Error: {}", err);

    Ok(())
}
