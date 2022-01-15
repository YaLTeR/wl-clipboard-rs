# Changelog

## Unreleased

- Split binaries from the main crate wl-clipboard-rs and create a new subcrate wl-clipboard-tools. (Closes: issue#15)

## v0.4.1 (1st Sep 2020)

- Updated `nix` to 0.18 and `wayland-rs` to 0.27.

## v0.4.0 (13th Dec 2019)

- **Breaking** Copying in non-foreground mode no longer forks (which was
  **unsafe** in multi-threaded programs). Instead, it spawns a background
  thread to serve copy requests.
- Added `copy::prepare_copy()` and `copy::prepare_copy_multi()` (and respective
  functions in `copy::Options`) to accommodate workflows which depended on the
  forking behavior, such as `wl-copy`. See `wl-copy` for example usage.
- **Breaking** Changed `copy::Source` and `copy::Seat` to own the contained
  data rather than borrow it. As a consequence, those types, as well as
  `copy::MimeSource` and `copy::Options`, have dropped their lifetime generic
  parameter.

### v0.3.1 (27th Nov 2019)

- Reduced the `wl_seat` version requirement from 6 to 2.
- Added `copy::copy_multi()` for offering multiple data sources under multiple
  different MIME types.

## v0.3.0 (4th Apr 2019)

- **Breaking** Moved `ClipboardType` into `copy::` and `paste::`.
- **Breaking** Renamed `utils::Error` into `utils::CopyDataError`.
- Added `copy::ClipboardType::Both` for operating both clipboards at once.
- Added `utils::is_primary_selection_supported()`.
- [wl-copy]: added `--regular`, which, when set together with `--primary`,
  makes `wl-copy` operate on both clipboards at once.

## v0.2.0 (17th Feb 2019)

- **Breaking** Changed `copy::Options::paste_once` to `serve_requests` which
  allows to specify the number of paste requests to serve.
- Marked `copy::Seat` and `copy::Options` as `Copy`.
- Updated `data-control`, it's now merged into `wlr-protocols` so no further
  changes without a version bump.
- [wl-copy, wl-paste]: replaced `env_logger` with `stderrlog` which made the
  binaries much smaller.
- Implemented `wl-clip`, a Wayland version of `xclip`.

## v0.1.0 (12th Feb 2019)

- Initial release.
