use thiserror::Error;

use crate::game::{castle_rights::CastleType, color::Color, piece::Piece, square::Square};

#[derive(Debug, Error)]
pub enum FENError {
    #[error("Error Parsing FEN String: {0}")]
    ParsingError(String),
    #[error("Missing Group {0} in FEN String")]
    MissingGroup(&'static str),
    #[error("Missing King in FEN String for color {0}")]
    MissingKing(Color),
}

#[derive(Debug, Error)]
pub enum MoveError {
    #[error("The source square {0} is empty")]
    EmptySource(Square),
    #[error("Invalid move")]
    InvalidMove,
    #[error("You cannot capture a King")]
    KingCapture,
    #[error("You cannot move to the same square")]
    SameSquare,
    #[error("No moves available from the source square")]
    NoMovesAvailable,
    #[error("Invalid piece move: {0} can not move from {1} to {2}")]
    InvalidPieceMove(Piece, Square, Square),
    #[error("It is not {0:?}'s turn to move")]
    WrongColor(Color),
    #[error("{1} does not have the right to castle {0:?}")]
    MissingCastleRight(CastleType, Color),
}

#[derive(Debug, Error)]
pub enum UndoMoveError {
    #[error("The is no game state record -> No moves can be undone")]
    NoMovesToUndo,
    #[error("The undo move is not the last move played on the board")]
    InvalidLastMove,
}
