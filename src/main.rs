mod errors;
mod state;
mod transaction;

use std::{fs::File, path::PathBuf};

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
/// The command-line arguments to the program
struct Args {
    #[clap(value_parser)]
    /// The input CSV file to process.
    input: PathBuf,
}

fn main() -> Result<(), errors::Error> {
    let args = Args::parse();
    let mut program_state = state::CurrentState::default();
    program_state.process_from_csv(File::open(args.input)?)?;
    program_state.into_csv(std::io::stdout())?;
    Ok(())
}
