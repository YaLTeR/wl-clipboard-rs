//! Watches the regular clipboard and prints each selection change until you press Enter.
//!
//! Run with `cargo run --example watch`, then copy some text in another application. Press Enter
//! to stop: that demonstrates cancelling a `Watcher` that is blocked waiting for the next event on
//! a different thread.
//!
//! Note that in `wl-paste` this pattern is not used because it's simpler to just allow a `SIGINT`
//! or similar to kill the process. In a more complex application this cancellation mechanism should
//! prove useful.

use std::error::Error;
use std::io::{self, BufRead, Read};
use std::thread;

use wl_clipboard_rs::paste::{ClipboardType, Seat};
use wl_clipboard_rs::watch::{ClipboardEvent, Watcher};

fn main() -> Result<(), Box<dyn Error>> {
    let mut watcher = Watcher::new(ClipboardType::Regular, Seat::Unspecified)?;
    let handle = watcher.cancel_handle();

    // The watcher below blocks in poll() waiting for the next selection. Pressing Enter writes to
    // the cancel pipe from this thread, which wakes that poll and makes next_event() return None.
    thread::spawn(move || {
        let mut line = String::new();
        let _ = io::stdin().lock().read_line(&mut line);
        handle.cancel();
    });

    println!("Watching the clipboard. Press Enter to stop.");

    while let Some((event, mut offer)) = watcher.next_event()? {
        match event {
            ClipboardEvent::Changed { mime_types } => {
                println!("changed: {} mime type(s) offered", mime_types.len());
                for mime_type in mime_types {
                    if mime_type.starts_with("text/") {
                        let mut pipe = offer.receive(&mime_type)?;
                        let mut bytes = Vec::new();
                        pipe.read_to_end(&mut bytes)?;
                        println!("- {mime_type}: {}", String::from_utf8_lossy(&bytes));
                    } else {
                        println!("- {mime_type}");
                    }
                }
            }
            ClipboardEvent::Cleared => println!("cleared"),
        }
    }

    println!("Cancelled.");
    Ok(())
}
