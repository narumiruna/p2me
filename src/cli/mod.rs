mod api;
mod credentials;
mod input_file;

use std::{
    io::{self, Write},
    path::PathBuf,
    process::ExitCode,
};

use clap::{Parser, Subcommand, ValueEnum, builder::NonEmptyStringValueParser};

use self::api::Api;

#[derive(Parser)]
#[command(
    name = "p2me",
    version,
    about = "Find theorems and verify Lean submissions on Prove2me",
    after_help = "Examples:\n  p2me theorems search poincare\n  p2me verify status <submission-id>"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Find or inspect a theorem.
    #[command(subcommand)]
    Theorems(Theorems),
    /// Submit a Lean file or check a verification result.
    #[command(subcommand)]
    Verify(Verify),
}

#[derive(Subcommand)]
enum Theorems {
    /// Search for theorems by keyword.
    Search {
        #[arg(value_parser = NonEmptyStringValueParser::new())]
        query: String,
    },
    /// Get the formal statement and other details for a theorem.
    Get {
        #[arg(value_parser = NonEmptyStringValueParser::new())]
        theorem_id: String,
    },
}

#[derive(Subcommand)]
enum Verify {
    /// Upload a Lean file and return the submission receipt without waiting.
    Submit {
        #[arg(value_parser = NonEmptyStringValueParser::new())]
        theorem_id: String,
        file: PathBuf,
        /// Whether the file proves or disproves the target theorem.
        #[arg(long, value_enum, default_value = "prove")]
        proof_type: ProofType,
        /// Written explanation to include with the submission.
        #[arg(long)]
        explanation: Option<String>,
    },
    /// Fetch the current status of a submission once.
    Status {
        #[arg(value_parser = NonEmptyStringValueParser::new())]
        submission_id: String,
    },
}

#[derive(Clone, Copy, ValueEnum)]
enum ProofType {
    Prove,
    Disprove,
}

impl ProofType {
    fn as_str(self) -> &'static str {
        match self {
            Self::Prove => "prove",
            Self::Disprove => "disprove",
        }
    }
}

pub fn run(base_url: &str) -> ExitCode {
    let cli = Cli::parse();
    match execute(base_url, cli.command) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::FAILURE
        }
    }
}

fn execute(base_url: &str, command: Command) -> Result<(), String> {
    let api = Api::new(base_url)?;
    let request = match command {
        Command::Theorems(Theorems::Search { query }) => api.search(&query),
        Command::Theorems(Theorems::Get { theorem_id }) => api.theorem(&theorem_id)?,
        Command::Verify(Verify::Submit {
            theorem_id,
            file,
            proof_type,
            explanation,
        }) => api.submit(theorem_id, &file, proof_type.as_str(), explanation)?,
        Command::Verify(Verify::Status { submission_id }) => api.status(&submission_id),
    };
    let result = api.send(request, &credentials::read_key()?)?;
    let mut stdout = io::stdout().lock();
    serde_json::to_writer_pretty(&mut stdout, &result)
        .map_err(|_| "could not write the JSON result")?;
    writeln!(stdout).map_err(|_| "could not write the JSON result")?;
    stdout
        .flush()
        .map_err(|_| "could not write the JSON result")?;
    Ok(())
}
