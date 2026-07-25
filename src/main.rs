use bart::{BartClient, Route, StationEtd};
use clap::{Parser, Subcommand, ValueEnum};
use std::{
    process,
    time::{Duration, Instant},
};

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

/// Direction filter accepted by the CLI.
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

fn color_icon(rgb: (u8, u8, u8)) -> String {
    let (r, g, b) = rgb;
    format!("\x1b[38;2;{r};{g};{b}m●\x1b[0m")
}

fn dir_icon(dir: &bart::Direction) -> &'static str {
    match dir {
        bart::Direction::North => "↑",
        bart::Direction::South => "↓",
    }
}

fn print_routes(routes: &[Route], icons: bool) {
    for dir in [bart::Direction::North, bart::Direction::South] {
        let group: Vec<&Route> = routes.iter().filter(|r| r.direction == dir).collect();
        if group.is_empty() {
            continue;
        }
        if icons {
            println!("\n  {}", dir_icon(&dir));
        } else {
            println!("\n  {dir}");
        }
        for r in group {
            if icons {
                println!(
                    "    {}  {}",
                    r.rgb().map(color_icon).unwrap_or_default(),
                    r.name
                );
            } else {
                println!("    {:<8}  {}", r.color, r.name);
            }
        }
    }
    println!();
}

fn print_estimates(station_etds: &[StationEtd], icons: bool) {
    for stn in station_etds {
        println!("{} ({})", stn.name, stn.abbr);

        let groups = stn.by_direction();

        for (dir, group) in [
            (bart::Direction::North, groups.north),
            (bart::Direction::South, groups.south),
        ] {
            if group.is_empty() {
                continue;
            }
            if icons {
                println!("\n  {}", dir_icon(&dir));
            } else {
                println!("\n  {dir}");
            }
            for etd in group {
                let est = &etd.estimate[0];
                let times: Vec<String> =
                    etd.estimate.iter().map(|e| e.minutes.to_string()).collect();
                if icons {
                    println!(
                        "    {}  {:<26}  {}",
                        est.rgb().map(color_icon).unwrap_or_default(),
                        etd.destination,
                        times.join(", ")
                    );
                } else {
                    println!(
                        "    {:<8}  {:<26}  {}",
                        est.color,
                        etd.destination,
                        times.join(", ")
                    );
                }
            }
        }
        println!();
    }
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

async fn fetch_estimates(
    client: &BartClient,
    station: &str,
    dir: Option<&str>,
) -> bart::Result<Vec<StationEtd>> {
    match dir {
        Some(d) => client.estimates_filtered(station, d).await,
        None => client.estimates(station).await,
    }
}

async fn run() -> bart::Result<()> {
    let cli = Cli::parse();
    let client = BartClient::new();
    let icons = !cli.no_icons && !cli.json;

    match cli.command {
        Command::Stations => {
            let stations = client.stations().await?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&stations)?);
            } else {
                for s in &stations {
                    println!("{:<6}  {}  ({}, {})", s.abbr, s.name, s.city, s.state);
                }
            }
        }
        Command::Routes { direction } => {
            let mut routes = client.routes().await?;
            if let Some(ref d) = direction {
                let dir = d.as_model();
                routes.retain(|r| r.direction == dir);
            }
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&routes)?);
            } else {
                print_routes(&routes, icons);
            }
        }
        Command::Estimates {
            station,
            direction,
            watch,
        } => {
            let dir_str = direction.as_ref().map(|d| d.api_str());

            if let Some(interval_secs) = watch {
                let api_interval = Duration::from_secs(interval_secs);
                let mut last_fetch: Option<Instant> = None;
                let mut etds: Vec<StationEtd> = Vec::new();

                print!("\x1b[?1049h"); // enter alternate screen
                loop {
                    if last_fetch.is_none_or(|t| t.elapsed() >= api_interval) {
                        match fetch_estimates(&client, &station, dir_str).await {
                            Ok(data) => {
                                etds = data;
                                last_fetch = Some(Instant::now());
                            }
                            Err(e) => {
                                print!("\x1b[?1049l");
                                return Err(e);
                            }
                        }
                    }

                    let secs = last_fetch.unwrap().elapsed().as_secs();
                    let ago = if secs == 0 {
                        "just now".to_string()
                    } else {
                        format!("{secs}s ago")
                    };

                    print!("\x1b[2J\x1b[H");
                    println!("Updated {ago}\n");
                    if cli.json {
                        println!("{}", serde_json::to_string_pretty(&etds)?);
                    } else {
                        print_estimates(&etds, icons);
                    }

                    tokio::select! {
                        _ = tokio::signal::ctrl_c() => break,
                        _ = tokio::time::sleep(Duration::from_secs(1)) => {}
                    }
                }
                print!("\x1b[?1049l"); // restore original screen
            } else {
                let etds = fetch_estimates(&client, &station, dir_str).await?;
                if cli.json {
                    println!("{}", serde_json::to_string_pretty(&etds)?);
                } else {
                    print_estimates(&etds, icons);
                }
            }
        }
    }

    Ok(())
}
