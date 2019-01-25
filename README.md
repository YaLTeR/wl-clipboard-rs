# wl-clipboard-rs

A Rust remake of [wl-clipboard](https://github.com/bugaevc/wl-clipboard). Work in progress.

Points of interest:
- The crate code itself is 100% safe Rust (this doesn't include the dependencies).
- Pure Rust implementation by default; use the `native_lib` feature to link to `libwayland-client.so` for communication instead.
- Uses the `data-control` protocol from `wlroots` for regular clipboard and the `gtk-primary-selection` protocol (plus spawning a surface with the `layer-shell` protocol) for the "primary" clipboard.
