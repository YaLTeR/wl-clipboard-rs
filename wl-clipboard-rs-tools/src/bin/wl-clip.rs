use std::{
    env::args_os,
    ffi::OsString,
    fs::File,
    io::{stdout, Read, Write},
    process,
};

use anyhow::{Context, Error};
use nix::unistd::{fork, ForkResult};

use wl_clipboard_rs::{
    copy::{self, ServeRequests, Source},
    paste::{self, get_contents},
    utils::is_text,
};

#[derive(Clone, Copy, Eq, PartialEq)]
enum Verbosity {
    Silent,
    Quiet,
    Verbose,
}

struct Options {
    files: Vec<OsString>,
    out: bool,
    loops: usize,
    target: Option<String>,
    rmlastnl: bool,
    verbosity: Verbosity,
    primary: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self { files: Vec::new(),
               out: false,
               loops: 0,
               target: None,
               rmlastnl: false,
               verbosity: Verbosity::Silent,
               primary: true }
    }
}

impl Options {
    // Hand-rolled argument parser to match what xclip does.
    fn from_args() -> Result<Self, Error> {
        let mut opts = Options::default();

        enum Print {
            Help,
            Version,
        }

        let mut print = None;

        let mut args = args_os().skip(1).peekable();
        while let Some(arg) = args.next() {
            match arg.into_string() {
                Ok(arg) => {
                    macro_rules! parse {
                        ($longest:expr, $shortest:expr => $action:block) => (
                            if $longest.starts_with(&arg) && arg.starts_with($shortest) {
                                $action
                            }
                        );

                        ($longest:expr, $shortest:expr, $next:ident => $action:block) => (
                            parse!($longest, $shortest => {
                                if args.peek().is_some() {
                                    let $next = args.next().unwrap();
                                    $action
                                }

                                // Important: no continue here.
                            });
                        );

                        ($longest:expr, $shortest:expr => $action:stmt) => (
                            parse!($longest, $shortest => {
                                $action
                                continue;
                            })
                        );
                    }

                    parse!("-help", "-h"            => print = Some(Print::Help));
                    parse!("-version", "-vers"      => print = Some(Print::Version));
                    parse!("-out", "-o"             => opts.out = true);
                    parse!("-in", "-i"              => opts.out = false);
                    parse!("-rmlastnl", "-r"        => opts.rmlastnl = true);
                    parse!("-silent", "-si"         => opts.verbosity = Verbosity::Silent);
                    parse!("-quiet", "-q"           => opts.verbosity = Verbosity::Quiet);
                    parse!("-verbose", "-verb"      => opts.verbosity = Verbosity::Verbose);

                    parse!("-filter", "-f"          => {
                        // Not sure there's a good way to support this.
                        anyhow::bail!("Unsupported option: -filter");
                    });

                    parse!("-noutf8", "-n"          => {
                        anyhow::bail!("Unsupported option: -noutf8");
                    });

                    parse!("-display", "-d"         => {
                        anyhow::bail!("Unsupported option: -display");
                    });

                    parse!("-selection", "-se", val => {
                        match val.to_string_lossy().chars().next().unwrap_or('_') {
                            'c' => opts.primary = false,
                            'p' => opts.primary = true,
                            's' => anyhow::bail!("Unsupported option: -selection secondary"),
                            'b' => anyhow::bail!("Unsupported option: -selection buffer-cut"),
                            _ => {}
                        }

                        continue;
                    });

                    parse!("-loops", "-l", val      => {
                        if let Some(val) = val.into_string().ok().and_then(|x| x.parse().ok()) {
                            opts.loops = val;
                        }

                        continue;
                    });

                    parse!("-target", "-t", val     => {
                        if let Ok(val) = val.into_string() {
                            opts.target = Some(val);
                        } else {
                            anyhow::bail!("Unsupported option: -target <invalid UTF-8>");
                        }

                        continue;
                    });

                    opts.files.push(arg.into())
                }

                Err(arg) => opts.files.push(arg),
            }
        }

        // If help or version is requested, print that and exit.
        match print {
            Some(Print::Help) => {
                eprintln!(
                          "\
Usage: {} [OPTION] [FILE]...
Access Wayland clipboard for reading or writing, with an xclip interface.

  -i, -in          read text into the clipboard from the standard input or files (default)
  -o, -out         print the contents of the clipboard to standard output
  -l, -loops       number of paste requests to serve before exiting
  -h, -help        show this message
      -selection   clipboard type to access, \"primary\" (default) or \"clipboard\"
      -target      set the MIME type to request or set
      -rmlastnl    trim the last newline character
      -version     show version information
      -silent      output errors only, run in background (default)
      -quiet       run in foreground
      -verbose     run in foreground, show verbose messages

Unsupported xclip options:
  -d, -display
  -f, -filter
      -selection secondary, buffer-cut
      -noutf8",
                          args_os().next()
                                   .and_then(|x| x.into_string().ok())
                                   .unwrap_or_else(|| "wl-clip".to_string())
                );
                process::exit(0);
            }
            Some(Print::Version) => {
                eprintln!("wl-clip version {}", env!("CARGO_PKG_VERSION"));
                eprintln!("{}", env!("CARGO_PKG_AUTHORS"));
                process::exit(0);
            }
            None => {}
        }

        Ok(opts)
    }
}

impl From<Options> for copy::Options {
    fn from(x: Options) -> Self {
        let mut opts = copy::Options::new();
        opts.serve_requests(if x.loops == 0 {
                                ServeRequests::Unlimited
                            } else {
                                ServeRequests::Only(x.loops)
                            })
            .foreground(true) // We fork manually to support background mode.
            .clipboard(if x.primary {
                           copy::ClipboardType::Primary
                       } else {
                           copy::ClipboardType::Regular
                       })
            .trim_newline(x.rmlastnl);
        opts
    }
}

fn main() -> Result<(), Error> {
    // Parse command-line options.
    let mut options = Options::from_args()?;

    stderrlog::new().verbosity(if options.verbosity == Verbosity::Verbose { 2 } else { 1 })
                    .init()
                    .unwrap();

    if options.out {
        // Paste.
        let mime_type = match options.target.as_ref() {
            Some(target) => paste::MimeType::Specific(target),
            None => paste::MimeType::Text,
        };

        let clipboard_type = if options.primary {
            paste::ClipboardType::Primary
        } else {
            paste::ClipboardType::Regular
        };

        let (mut read, mime_type) = get_contents(clipboard_type, paste::Seat::Unspecified, mime_type)?;

        // Read the contents.
        let mut contents = vec![];
        read.read_to_end(&mut contents)
            .context("Couldn't read clipboard contents")?;

        // Remove the last newline character if needed.
        let last_character_is_newline = contents.last().map(|&c| c == b'\n').unwrap_or(false);
        if options.rmlastnl && is_text(&mime_type) && last_character_is_newline {
            contents.pop();
        }

        // Write everything to stdout.
        stdout().write_all(&contents)
                .context("Couldn't write contents to stdout")?;
    } else {
        // Copy.
        let data = if options.files.is_empty() {
            None
        } else {
            let mut data = vec![];

            for filename in &options.files {
                let mut file = File::open(filename).context(format!("Couldn't open {}", filename.to_string_lossy()))?;
                file.read_to_end(&mut data)?;
            }

            Some(data)
        };

        let source = if options.files.is_empty() {
            Source::StdIn
        } else {
            Source::Bytes(data.unwrap().into())
        };

        let mime_type = if let Some(mime_type) = options.target.take() {
            copy::MimeType::Specific(mime_type)
        } else {
            // xclip uses STRING in this case, but I believe this is better.
            // If it breaks anyone, it should be changed to Text or Specific("STRING").
            copy::MimeType::Autodetect
        };

        let foreground = options.verbosity != Verbosity::Silent;

        let prepared_copy = copy::Options::from(options).prepare_copy(source, mime_type)?;

        if foreground {
            prepared_copy.serve()?;
        } else {
            // SAFETY: We don't spawn any threads, so doing things after forking is safe.
            // TODO: is there any way to verify that we don't spawn any threads?
            if let ForkResult::Child = unsafe { fork() }.unwrap() {
                drop(prepared_copy.serve());
            }
        }
    }

    Ok(())
}
