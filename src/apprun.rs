fn main() {
    let here_dir = std::path::PathBuf::from(std::env::current_exe().unwrap());
    std::env::set_var("LD_LIBRARY_PATH", format!("{}/usr/lib/:{}/usr/lib/i386-linux-gnu/:{}/usr/lib/x86_64-linux-gnu/:{}/usr/lib32/:{}/usr/lib64/:{}/lib/:{}/lib/i386-linux-gnu/:{}/lib/x86_64-linux-gnu/:{}/lib32/:{}/lib64/:{}", here_dir.parent().unwrap().display(), here_dir.parent().unwrap().display(), here_dir.parent().unwrap().display(), here_dir.parent().unwrap().display(), here_dir.parent().unwrap().display(), here_dir.parent().unwrap().display(), here_dir.parent().unwrap().display(), here_dir.parent().unwrap().display(), here_dir.parent().unwrap().display(), here_dir.parent().unwrap().display(), std::env::var("LD_LIBRARY_PATH").unwrap_or(String::new())));
    std::env::set_var(
        "XDG_DATA_DIRS",
        format!(
            "XDG_DATA_DIRS={}:{}",
            here_dir.parent().unwrap().join("usr/share").display(),
            std::env::var("XDG_DATA_DIRS").unwrap_or(String::new())
        ),
    );
    let err = exec::execvp(
        here_dir.parent().unwrap().join("usr/bin/bin"),
        std::env::args(),
    );
    eprintln!("Error: {}", err);
    std::process::exit(1);
}

// .env(
//            "LD_LIBRARY_PATH",
//            if let Ok(env) = std::env::var("LD_LIBRARY_PATH") {
//                args.map(|i| i + ":").collect::<String>() + &env
//            } else {
//                args.map(|i| i + ":").collect::<String>()
//            },
//        )
