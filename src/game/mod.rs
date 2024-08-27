pub mod bit_manipulation;
pub mod board;
pub mod castle_rights;
pub mod color;
pub mod move_notation;
pub mod parser;
pub mod piece;
pub mod piece_type;
pub mod square;

// Re-export the most important types
pub use board::game_result::GameResult;
pub use board::move_gen::MoveGeneration;
pub use board::Board;
pub use color::Color;
pub use move_notation::Move;
pub use piece::Piece;
pub use piece_type::PieceType;
pub use square::Square;
