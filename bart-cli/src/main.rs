mod cmd;

use bart::BartClient;
use clap::{Parser, Subcommand, ValueEnum};
use std::process;

#[derive(Parser)]
#[command(
    name = "bart",
    about = "CLI for the BART API — stations, routes, and real-time departures"
)]
struct Cli {
    /// Output raw JSON (useful for piping to jq, etc.)
    #[arg(long, global = true)]
    json: bool,
    /// Disable direction and color icons; show plain text instead
    #[arg(long = "no-icons", global = true)]
    no_icons: bool,
    #[command(subcommand)]
    command: Command,
}

#[derive(Clone, ValueEnum)]
enum Direction {
    #[value(alias = "n")]
    North,
    #[value(alias = "s")]
    South,
}

impl Direction {
    fn api_str(&self) -> &'static str {
        match self {
            Direction::North => "n",
            Direction::South => "s",
        }
    }

    fn as_model(&self) -> bart::Direction {
        match self {
            Direction::North => bart::Direction::North,
            Direction::South => bart::Direction::South,
        }
    }
}

#[derive(Subcommand)]
enum Command {
    /// List all BART stations
    Stations,
    /// List all BART routes
    Routes {
        /// Filter by direction
        #[arg(short = 'd', long = "direction")]
        direction: Option<Direction>,
    },
    /// Real-time departure estimates for a station
    Estimates {
        /// Station abbreviation (e.g. 12TH, EMBR, GLEN)
        station: String,
        /// Filter by direction
        #[arg(short = 'd', long = "direction")]
        direction: Option<Direction>,
        /// Refresh every N seconds until Ctrl-C (default: 60)
        #[arg(
            short = 'w',
            long = "watch",
            value_name = "SECS",
            num_args = 0..=1,
            default_missing_value = "60"
        )]
        watch: Option<u64>,
    },
}

#[tokio::main]
async fn main() {
    // Silently exit on broken pipe (e.g. `bart ... | head`).
    // Rust sets SIGPIPE to SIG_IGN by default, causing println! to panic on EPIPE.
    #[cfg(unix)]
    // SAFETY: called before any threads are spawned.
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }

    if let Err(e) = run().await {
        match e {
            bart::Error::Api(msg) => eprintln!("error: {msg}"),
            e => eprintln!("error: {e}"),
        }
        process::exit(1);
    }
}

async fn run() -> bart::Result<()> {
    let cli = Cli::parse();
    let client = BartClient::new();
    let icons = !cli.no_icons && !cli.json;

    match cli.command {
        Command::Stations => {
            let stations = client.stations().await?;
            cmd::stations::run(&stations, cli.json)?;
        }
        Command::Routes { direction } => {
            let mut routes = client.routes().await?;
            if let Some(ref d) = direction {
                routes.retain(|r| r.direction == d.as_model());
            }
            cmd::routes::run(&routes, cli.json, icons)?;
        }
        Command::Estimates {
            station,
            direction,
            watch,
        } => {
            let dir_str = direction.as_ref().map(|d| d.api_str());
            cmd::estimates::run(&client, &station, dir_str, watch, cli.json, icons).await?;
        }
    }

    Ok(())
}
