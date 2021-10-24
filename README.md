# Cargo AppImage

This a cargo program that allows you to convert your Rust programs into AppImages.

## Installation

1.  Make sure that `appimagetool` is in your path. It can be downloaded from [here](https://appimage.github.io/appimagetool/)
2.  Install this program with

```shell
cargo install cargo-appimage
```

3.  `cd` inside of the root directory of your crate and create an icon called **icon.png**
    1.  Note this can simply be an empty file for development. In fact an empty file is generated if you forget to make one.

4.  (optional) create a section in your Cargo.toml similar to the following
    with any additional assets to add to the AppImg:
    ```toml
    [package.metadata.appimage]
    assets = ["images", "sounds"]
    ```

5.  (optional) If you are using external crates that use other programs or are not written in pure rust, you may want to check if you need to embed some shared libraries into your AppImage:

    1.  Running `cargo appimage` with this option in your Cargo.toml will automatically make a libs folder and put all of the shared objects your rust program uses in their respective directories.

    ```toml
    [package.metadata.appimage]
    auto_link = true
    ```

    2.  AppImages aren't supposed to have EVERY library that your executable links to inside of the AppImage, so delete the libraries that you expect will be on every linux system (libc, libgcc, libpthread, ld-linux, libdl, etc.)

    3.  Remove the Cargo.toml option above and run `cargo appimage` again, now only the libs you want should be embedded in the Appimage

6.  run this command

    ```shell
    cargo appimage
    ```

    1.  Note all arguments passed into cargo-appimage are redirected to cargo

    ```shell
    cargo appimage --features=min
    ```
