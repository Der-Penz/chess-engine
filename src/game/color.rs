#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    White = 0,
    Black = 1,
}

impl Default for Color {
    fn default() -> Self {
        Color::White
    }
}

impl Color {
    pub fn from_char(c: char) -> Color {
        match c.to_ascii_lowercase() {
            'w' => Color::White,
            'b' => Color::Black,
            _ => panic!("Invalid char for color"),
        }
    }

    pub fn opposite(&self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }

    pub fn transform_str(&self, value: &str) -> String {
        match self {
            Color::White => value.to_ascii_uppercase(),
            Color::Black => value.to_ascii_lowercase(),
        }
    }

    pub fn transform_char(&self, value: &char) -> char {
        match self {
            Color::White => value.to_ascii_uppercase(),
            Color::Black => value.to_ascii_lowercase(),
        }
    }

    /// Returns the perspective of the color. White is 1, Black is -1.
    pub fn perspective(&self) -> i8 {
        match self {
            Color::White => 1,
            Color::Black => -1,
        }
    }

    pub fn pawn_rank(&self) -> u8 {
        match self {
            Color::White => 1,
            Color::Black => 6,
        }
    }

    pub fn promotion_rank(&self) -> u8 {
        match self {
            Color::White => 7,
            Color::Black => 0,
        }
    }

    pub fn relative_sq(&self, sq: u8) -> u8 {
        match self {
            Color::White => sq,
            Color::Black => 63 - sq,
        }
    }
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", char::from(*self))
    }
}

impl From<char> for Color {
    fn from(c: char) -> Self {
        if c.is_uppercase() {
            Color::White
        } else {
            Color::Black
        }
    }
}

impl From<Color> for char {
    fn from(value: Color) -> Self {
        match value {
            Color::White => 'w',
            Color::Black => 'b',
        }
    }
}

impl<T, const N: usize> std::ops::Index<Color> for [T; N] {
    type Output = T;

    fn index(&self, index: Color) -> &Self::Output {
        &self[index as usize]
    }
}

impl<T, const N: usize> std::ops::IndexMut<Color> for [T; N] {
    fn index_mut(&mut self, index: Color) -> &mut Self::Output {
        &mut self[index as usize]
    }
}
