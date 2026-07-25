pub mod estimates;
pub mod routes;
pub mod stations;

pub fn color_icon(rgb: (u8, u8, u8)) -> String {
    let (r, g, b) = rgb;
    format!("\x1b[38;2;{r};{g};{b}m●\x1b[0m")
}

pub fn dir_icon(dir: &bart::Direction) -> &'static str {
    match dir {
        bart::Direction::North => "↑",
        bart::Direction::South => "↓",
    }
}
