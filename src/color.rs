pub enum Color
{
    White,
    Black,
    Red,
    Green,
    Blue,
    Yellow,
    Cyan,
    Magenta,
}

impl Color
{
    pub fn to_ansi_escape_code(&self) -> &'static str
    {
        match self
        {
            Color::White => "\x1b[37m",
            Color::Black => "\x1b[30m",
            Color::Red => "\x1b[31m",
            Color::Green => "\x1b[32m",
            Color::Blue => "\x1b[34m",
            Color::Yellow => "\x1b[33m",
            Color::Cyan => "\x1b[36m",
            Color::Magenta => "\x1b[35m",
        }
    }
}

