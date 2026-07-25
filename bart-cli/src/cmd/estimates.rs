use std::time::{Duration, Instant};

use bart::{BartClient, Direction, StationEtd};

use crate::cmd::{color_icon, dir_icon};

pub async fn fetch(
    client: &BartClient,
    station: &str,
    dir: Option<&str>,
) -> bart::Result<Vec<StationEtd>> {
    match dir {
        Some(d) => client.estimates_filtered(station, d).await,
        None => client.estimates(station).await,
    }
}

pub fn print(etds: &[StationEtd], icons: bool) {
    for stn in etds {
        println!("{} ({})", stn.name, stn.abbr);
        let groups = stn.by_direction();
        for (dir, group) in [
            (Direction::North, groups.north),
            (Direction::South, groups.south),
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

pub async fn run(
    client: &BartClient,
    station: &str,
    dir: Option<&str>,
    watch: Option<u64>,
    json: bool,
    icons: bool,
) -> bart::Result<()> {
    if let Some(interval_secs) = watch {
        let api_interval = Duration::from_secs(interval_secs);
        let mut last_fetch: Option<Instant> = None;
        let mut etds: Vec<StationEtd> = Vec::new();

        print!("\x1b[?1049h"); // enter alternate screen
        loop {
            if last_fetch.is_none_or(|t| t.elapsed() >= api_interval) {
                match fetch(client, station, dir).await {
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
            if json {
                println!("{}", serde_json::to_string_pretty(&etds)?);
            } else {
                print(&etds, icons);
            }

            tokio::select! {
                _ = tokio::signal::ctrl_c() => break,
                _ = tokio::time::sleep(Duration::from_secs(1)) => {}
            }
        }
        print!("\x1b[?1049l"); // restore original screen
    } else {
        let etds = fetch(client, station, dir).await?;
        if json {
            println!("{}", serde_json::to_string_pretty(&etds)?);
        } else {
            print(&etds, icons);
        }
    }
    Ok(())
}
