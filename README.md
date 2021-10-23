# Cargo AppImage

This a cargo program that allows you to convert your Rust programs into AppImages.

## Installation

1.  Make sure that `appimagetool` is in your path. It can be downloaded from [here](https://appimage.github.io/appimagetool/)
2.  Install this program with

```shell
cargo install cargo-appimage
```

3. `cd` inside of the root directory of your crate and create an icon called **icon.png**
   1. Note this can simply be an empty file for development. In fact an empty file is generated if you forget to make one.
4. (optional) create a section in your Cargo.toml similar to the following
    with any additional assets to add to the AppImg:
    ```toml
    [package.metadata.appimage]
    assets = ["images", "sounds"]
    ```
6. run this command

    ```shell
    cargo appimage
    ```

    1. Note all arguments passed into cargo-appimage are redirected to cargo

    ```shell
    cargo appimage --features=min
    ```

