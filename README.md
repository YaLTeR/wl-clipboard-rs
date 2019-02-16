# wl-clipboard-rs

[![crates.io](https://img.shields.io/crates/v/wl-clipboard-rs.svg)](https://crates.io/crates/wl-clipboard-rs)
[![Build Status](https://travis-ci.com/YaLTeR/wl-clipboard-rs.svg?branch=master)](https://travis-ci.com/YaLTeR/wl-clipboard-rs)

[Documentation](https://yalter.github.io/wl-clipboard-rs/wl_clipboard_rs/)

A safe Rust crate for working with the Wayland clipboard.

This crate is intended to be used by terminal applications, clipboard managers and other
utilities which don't spawn Wayland surfaces (windows). If your application has a window,
please use the appropriate Wayland protocols for interacting with the Wayland clipboard
(`wl_data_device` from the core Wayland protocol, the `primary_selection` protocol for the
primary selection), for example via the
[smithay-clipboard](https://github.com/Smithay/smithay-clipboard) crate.

The protocol used for clipboard interaction is `data-control` from
[wlroots](https://github.com/swaywm/wlr-protocols). When using the regular clipboard, the
compositor must support the first version of the protocol. When using the "primary" clipboard,
the compositor must support the second version of the protocol (or higher).

For example applications using these features, see `src/bin/wl_copy.rs` and
`src/bin/wl_paste.rs` which implement terminal apps similar to
[wl-clipboard](https://github.com/bugaevc/wl-clipboard) or `src/bin/wl_clip.rs` which
implements a Wayland version of `xclip`.

The Rust implementation of the Wayland client is used by default; use the `native_lib` feature
to link to `libwayland-client.so` for communication instead. A `dlopen` feature is also
available for loading `libwayland-client.so` dynamically at runtime rather than linking to it.

The code of the crate itself (and the code of the example utilities) is 100% safe Rust. This
doesn't include the dependencies.

## Examples

Copying to the regular clipboard:
```rust
use wl_clipboard_rs::copy::{MimeType, Options, Source};

let opts = Options::new();
opts.copy(Source::Bytes("Hello world!".as_bytes()), MimeType::Autodetect)?;
```

Pasting plain text from the regular clipboard:
```rust
use std::io::Read;
use wl_clipboard_rs::{paste::{get_contents, Error, MimeType, Seat}, ClipboardType};

let result = get_contents(ClipboardType::Regular, Seat::Unspecified, MimeType::Text);
match result {
    Ok((mut pipe, _)) => {
        let mut contents = vec![];
        pipe.read_to_end(&mut contents)?;
        println!("Pasted: {}", String::from_utf8_lossy(&contents));
    }

    Err(Error::NoSeats) | Err(Error::ClipboardEmpty) | Err(Error::NoMimeType) => {
        // The clipboard is empty or doesn't contain text, nothing to worry about.
    }

    Err(err) => Err(err)?
}
```

### Terminal applications

- `wl-paste`: implements `wl-paste` from [wl-clipboard](https://github.com/bugaevc/wl-clipboard).
- `wl-copy`: implements `wl-copy` from [wl-clipboard](https://github.com/bugaevc/wl-clipboard).
- `wl-clip`: a Wayland version of `xclip`.

Stuff that would be neat to add:
- Utility that mimics `xsel` commandline flags.

License: MIT/Apache-2.0
