use iced::Font;

pub const ICON_FONT: Font = Font::with_name("bootstrap-icons");

pub enum IconChar {
    Rocket,
    InfoFill,
    InfoCircle,
    GitHub,
    Shuffle,
    X,
    Download,
}

impl IconChar {
    pub fn as_str(&self) -> char {
        match self {
            IconChar::Rocket => '\u{f845}',
            IconChar::InfoFill => '\u{f430}',
            IconChar::InfoCircle => '\u{f431}',
            IconChar::GitHub => '\u{f3ed}',
            IconChar::Shuffle => '\u{f544}',
            IconChar::X => '\u{f62A}',
            IconChar::Download => '\u{f30A}',
        }
    }
}
