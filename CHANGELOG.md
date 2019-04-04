# Changelog

## Unreleased

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
