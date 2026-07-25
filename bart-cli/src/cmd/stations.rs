use bart::Station;

pub fn run(stations: &[Station], json: bool) -> bart::Result<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(stations)?);
    } else {
        for s in stations {
            println!("{:<6}  {}  ({}, {})", s.abbr, s.name, s.city, s.state);
        }
    }
    Ok(())
}
