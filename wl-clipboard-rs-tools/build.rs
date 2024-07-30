#[path = "src/wl_copy.rs"]
mod wl_copy;

#[path = "src/wl_paste.rs"]
mod wl_paste;

use std::error::Error;
use std::fs;
use std::path::PathBuf;

use clap::{Command, CommandFactory};
use clap_complete::generate_to;
use clap_complete::Shell::{Bash, Fish, Zsh};
use clap_mangen::Man;

fn generate_man_pages(name: &str, cmd: &Command) -> Result<(), Box<dyn Error>> {
    let man_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../target/man");
    let mut buffer = Vec::default();

    Man::new(cmd.clone()).render(&mut buffer)?;
    fs::create_dir_all(&man_dir)?;
    fs::write(man_dir.join(name.to_owned() + ".1"), buffer)?;

    Ok(())
}

fn generate_shell_completions(name: &str, mut cmd: Command) -> Result<(), Box<dyn Error>> {
    let comp_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../target/completions");

    fs::create_dir_all(&comp_dir)?;

    for shell in [Bash, Fish, Zsh] {
        generate_to(shell, &mut cmd, name, &comp_dir)?;
    }

    Ok(())
}

fn generate(name: &str, mut cmd: Command) {
    cmd.set_bin_name(name);

    if let Err(err) = generate_man_pages(name, &cmd) {
        println!("cargo::warning=error generating man page for {name}: {err}");
    }

    if let Err(err) = generate_shell_completions(name, cmd) {
        println!("cargo::warning=error generating completions for {name}: {err}");
    }
}

fn main() {
    generate("wl-copy", wl_copy::Options::command());
    generate("wl-paste", wl_paste::Options::command());
}
