use bart::{Direction, Route};

use crate::cmd::{color_icon, dir_icon};

pub fn run(routes: &[Route], json: bool, icons: bool) -> bart::Result<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(routes)?);
        return Ok(());
    }
    for dir in [Direction::North, Direction::South] {
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
    Ok(())
}
