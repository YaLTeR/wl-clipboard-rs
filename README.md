# wl-clipboard-rs

A safe Rust crate for working with the Wayland clipboard.

This crate is intended to be used by terminal applications, clipboard managers and other
utilities which don't spawn Wayland surfaces (windows). If your application has a window,
please use the appropriate Wayland protocols for interacting with the Wayland clipboard
(`wl_data_device` from the core Wayland protocol, the `primary_selection` protocol for the
primary selection).

The protocol used for clipboard interaction is `data-control` from
[wlroots](https://github.com/swaywm/wlr-protocols). When using the regular clipboard, the
compositor must support the first version of the protocol. When using the "primary" clipboard,
the compositor must support the second version of the protocol (or higher).

For example applications using these features, see `bin/wl_copy.rs` and `bin/wl_paste.rs` which
implement terminal apps similar to [wl-clipboard](https://github.com/bugaevc/wl-clipboard).

The Rust implementation of the Wayland client is used by default; use the `native_lib` feature
to link to `libwayland-client.so` for communication instead.

The code of the crate itself (and the code of the example utilities) is 100% safe Rust. This
doesn't include the dependencies.

### Status

`wl-paste`:
- Mostly done.
- TODO: output MIME type detection.
- TODO: smart MIME type selection (use `text/plain;charset=utf-8` if it's available and the MIME type is unspecified, etc.).
- TODO: tests.

`wl-copy`:
- Mostly done.
- TODO: MIME type inference.
- TODO: tests.

Stuff that would be neat to add:
- Utilities that mimic `xclip` and `xsel` commandline flags.

License: MIT/Apache-2.0
