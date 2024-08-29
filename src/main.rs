use std::{path::PathBuf, process::ExitCode};

use clap::Parser;
use expand_mod::{expand_from_path, ExpandError};

#[derive(clap::Parser)]
struct Args {
    #[clap(long)]
    clipboard: bool,

    files: Vec<PathBuf>,
}

fn main() -> ExitCode {
    match run() {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            e.show();
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), ExpandError> {
    let args = Args::parse();
    let mut text = String::new();
    for file in &args.files {
        text.push_str(&expand_from_path(file, true)?);
    }
    if args.clipboard {
        arboard::Clipboard::new()?.set_text(text)?;
    } else {
        println!("{text}");
    }
    Ok(())
}
