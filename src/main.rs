use std::{path::Path, process::ExitCode};

use expand_mod::{expand_from_path, ExpandError};

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
    for arg in std::env::args().skip(1) {
        let path = Path::new(&arg);
        let code = expand_from_path(path, true)?;
        println!("{code}");
    }
    Ok(())
}
