use clap::Parser;
use clap::ValueEnum;
use std::process::ExitCode;
use storm_events::atcf::ATCFFileDeck;
use storm_events::storm_event::StormEventBuilder;

#[derive(Parser, Debug)]
#[command(author, about, long_about = None)]
struct Cli {
    #[clap(help = "Can be NameYear (e.g. Sandy2012) or NHC code (e.g. AL182012)")]
    storm_id: String,
    file_deck: FileDeckKind,
}

#[derive(ValueEnum, Clone, Debug)]
enum FileDeckKind {
    ADVISORY,
    BEST,
    FIXED,
}

impl FileDeckKind {
    fn to_atcf_file_deck(&self) -> ATCFFileDeck {
        match self {
            FileDeckKind::ADVISORY => ATCFFileDeck::ADVISORY,
            FileDeckKind::BEST => ATCFFileDeck::BEST,
            FileDeckKind::FIXED => ATCFFileDeck::FIXED,
        }
    }
}

fn entrypoint() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let storm_event = StormEventBuilder::default()
        .file_deck(&cli.file_deck.to_atcf_file_deck())
        .storm_id(&cli.storm_id)
        .build()?;
    dbg!(storm_event);
    Ok(())
}

fn main() -> ExitCode {
    let exit_code = match entrypoint() {
        Err(e) => {
            eprintln!("Error: {}", e);
            return ExitCode::FAILURE;
        }
        Ok(_) => ExitCode::SUCCESS,
    };
    exit_code
}
