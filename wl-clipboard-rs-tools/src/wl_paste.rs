use clap::Parser;

#[derive(Parser)]
#[command(
    name = "wl-paste",
    version,
    about = "Paste clipboard contents on Wayland."
)]
pub struct Options {
    /// List the offered MIME types instead of pasting
    #[arg(long, short)]
    pub list_types: bool,

    /// Use the "primary" clipboard
    ///
    /// Pasting to the "primary" clipboard requires the compositor to support the data-control
    /// protocol of version 2 or above.
    #[arg(long, short)]
    pub primary: bool,

    /// Do not append a newline character
    ///
    /// By default the newline character is appended automatically when pasting text MIME types.
    #[arg(long, short, conflicts_with = "list_types")]
    pub no_newline: bool,

    /// Pick the seat to work with
    ///
    /// By default the seat used is unspecified (it depends on the order returned by the
    /// compositor). This is perfectly fine when only a single seat is present, so for most
    /// configurations.
    #[arg(long, short)]
    pub seat: Option<String>,

    /// Request the given MIME type instead of inferring the MIME type
    ///
    /// As a special case, specifying "text" will look for a number of plain text types,
    /// prioritizing ones that are known to give UTF-8 text.
    #[arg(
        name = "MIME/TYPE",
        long = "type",
        short = 't',
        conflicts_with = "list_types"
    )]
    pub mime_type: Option<String>,

    /// Enable verbose logging
    #[arg(long, short, action = clap::ArgAction::Count)]
    pub verbose: u8,
}
