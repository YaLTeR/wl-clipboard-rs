# wl-clipboard-rs

A Rust remake of [wl-clipboard](https://github.com/bugaevc/wl-clipboard). Work in progress.

Points of interest:
- The crate code itself is 100% safe Rust (this doesn't include the dependencies).
- Pure Rust implementation by default; use the `native_lib` feature to link to `libwayland-client.so` for communication instead.
- Protocols used:
  - `data-control` from `wlroots` for regular clipboard,
  - `gtk-primary-selection` or `primary-selection-v1` (plus spawning a surface with the `layer-shell` protocol) for the "primary" clipboard.

### Status

`wl-paste`:
- Mostly done.
- TODO: output MIME type detection.
- TODO: smart MIME type selection (use `text/plain;charset=utf-8` if it's available and the MIME type is unspecified, etc.).
- TODO: proper error handling (right now it just panics on any error).
- TODO: tests.

`wl-copy`:
- Mostly done for the regular clipboard.
- TODO: primary selection. Either figure out and implement spawning a surface and using the primary selection protocols, or just wait for `data-control` to add primary selection support and implement that.
- TODO: MIME type inference.
- TODO: proper error handling (right now it just panics on any error).
- TODO: tests.

Stuff that would be neat to add:
- Utilities that mimic `xclip` and `xsel` commandline flags.
