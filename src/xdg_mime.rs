use std::{
    path::Path,
    process::Command,
};
use log::debug;

const NEWLINE_SUFFIX: &'static [u8] = b"\n";

pub fn mime_from_filename<P: AsRef<Path>>(p: P) -> String {
    let p = p.as_ref();
    debug!("Detecting mime type for {}", p.display());
    let output = Command::new("xdg-mime")
            .args(&["query".as_ref(), "filetype".as_ref(), p])
            .output()
            .expect("Failed to detect mime type");
    let unstripped = output.stdout.as_slice();
    let stripped = if unstripped.ends_with(NEWLINE_SUFFIX) {
        &unstripped[0..unstripped.len() - NEWLINE_SUFFIX.len()]
    } else {
        unstripped
    };
    String::from_utf8_lossy(stripped).into_owned()
}
