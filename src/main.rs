use std::{path::PathBuf, process::ExitCode};

use clap::Parser;
use expand_mod::{expand_from_path, ExpandError};

/// Expand `mod module_name;` in `.rs` files and combine the module tree consisting of multiple files into a single file.
#[derive(clap::Parser)]
struct Args {
    /// Copy the result to the clipboard instead of stdout.
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
        let root = file.canonicalize()?.parent().unwrap().to_path_buf();
        text.push_str(&expand_from_path(&root, file, true)?);
    }
    if args.clipboard {
        arboard::Clipboard::new()?.set_text(text)?;
    } else {
        println!("{text}");
    }
    Ok(())
}
