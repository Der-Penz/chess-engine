use super::Square;

#[derive(FromPrimitive, Clone, Copy, PartialEq)]
pub enum Color {
    WHITE = 0,
    BLACK = 1,
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

impl Color {
    pub fn opposite(&self) -> Color {
        match self {
            Color::WHITE => Color::BLACK,
            Color::BLACK => Color::WHITE,
        }
    }

    pub fn transform_str(&self, value: &str) -> String {
        match self {
            Color::WHITE => value.to_ascii_uppercase(),
            Color::BLACK => value.to_ascii_lowercase(),
        }
    }

    pub fn transform_char(&self, value: &char) -> char {
        self.transform_str(value.to_string().as_ref())
            .chars()
            .next()
            .unwrap()
    }

    pub fn relative_sq(&self, sq: Square) -> Square {
        match self {
            Color::WHITE => sq,
            Color::BLACK => {
                let rank = 7 - sq.rank();
                Square::from(rank * 8 + sq.file())
            }
        }
    }

    pub fn relative(&self, sq: u8) -> u8 {
        match self {
            Color::WHITE => sq,
            Color::BLACK => {
                let rank = 7 - (sq / 8);
                (rank * 8) + (sq % 8)
            }
        }
    }
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let color = match self {
            Color::WHITE => "w",
            Color::BLACK => "b",
        };
        write!(f, "{}", color)
    }
}

impl std::fmt::Debug for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let color = match self {
            Color::WHITE => "⬤",
            Color::BLACK => "◯",
        };
        write!(f, "{}", color)
    }
}

impl Default for Color {
    fn default() -> Self {
        Color::WHITE
    }
}
