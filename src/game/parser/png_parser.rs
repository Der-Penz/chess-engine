use thiserror::Error;

use crate::game::{
    board::move_gen::MoveGeneration, castle_rights::CastleType, Board, Move, PieceType, Square,
};

pub struct PGNParser();

impl PGNParser {
    /// Convert the move to SAN (Standard Algebraic Notation).
    pub fn move_as_san(mov: &Move, board: &mut Board) -> Result<String, ToSANError> {
        let legal_moves = MoveGeneration::generate_legal_moves(board);

        if !legal_moves.has(mov) {
            return Err(ToSANError::InvalidMove);
        }

        if mov.is_null() {
            return Ok("null".to_string());
        }

        if let Some(castle_type) = mov.flag().castle_side() {
            return Ok(castle_type.to_string());
        }

        let source = mov.source();
        let dest = mov.dest();
        let moved_piece = board
            .get_sq_piece(source)
            .ok_or(ToSANError::InvalidMove)?
            .ptype();
        let capture_piece = board.get_sq_piece(dest);

        let mut san = moved_piece.to_string().to_ascii_uppercase();

        if moved_piece != PieceType::Pawn && moved_piece != PieceType::King {
            let same_piece_moves = legal_moves
                .iter()
                .filter(|m| {
                    m.source() != source
                        && m.dest() == dest
                        && board.get_sq_piece(m.source()).unwrap().ptype() == moved_piece
                })
                .collect::<Vec<_>>();

            if same_piece_moves.len() > 1 {
                let same_rank = same_piece_moves
                    .iter()
                    .any(|m| m.source().rank() == source.rank());
                let same_file = same_piece_moves
                    .iter()
                    .any(|m| m.source().file() == source.file());

                // piece on the same rank push the file
                if same_rank {
                    san.push_str(&source.file_str());
                // piece on the same file push the rank
                } else if same_file {
                    san.push_str(&source.rank_str());
                }
            }
        }

        if capture_piece.is_some() {
            if moved_piece == PieceType::Pawn {
                san.push_str(&source.file_str());
            }
            san.push('x');
        } else if mov.flag().is_en_passant() {
            san.push_str(&source.file_str());
            san.push('x');
        }

        san.push_str(&dest.to_string());

        if let Some(promotion_piece) = mov.flag().promotion_type() {
            san.push('=');
            san.push_str(&promotion_piece.to_string().to_ascii_uppercase());
        }

        board
            .make_move(mov, true, false)
            .map_err(|_| ToSANError::MoveMakeError)?;

        if board.in_check() {
            let legal_response_moves = MoveGeneration::generate_legal_moves(board);
            if legal_response_moves.is_empty() {
                san.push('#');
            } else {
                san.push('+');
            }
        }

        board
            .undo_move(mov, true)
            .map_err(|_| ToSANError::MoveUndoError)?;

        Ok(san)
    }

    /// Get the move from SAN (Standard Algebraic Notation).
    pub fn move_from_san(san: &str, board: &mut Board) -> Result<Move, FromSANError> {
        if san == "null" {
            return Ok(Move::null());
        }
        //remove unnecessary characters
        let cleaned_san = &san.replace("+", "").replace("#", "").replace("x", "")[..];
        let legal_moves = MoveGeneration::generate_legal_moves(board);

        // Check if the move is a castle move
        if cleaned_san == "OO" {
            return legal_moves
                .iter()
                .find(|m| {
                    m.flag()
                        .castle_side()
                        .is_some_and(|c| c == CastleType::KingSide)
                })
                .ok_or(FromSANError::InvalidMove)
                .cloned();
        }

        if cleaned_san == "OOO" {
            return legal_moves
                .iter()
                .find(|m| {
                    m.flag()
                        .castle_side()
                        .is_some_and(|c| c == CastleType::QueenSide)
                })
                .ok_or(FromSANError::InvalidMove)
                .cloned();
        }

        //must be at least 2 characters
        if cleaned_san.len() < 2 {
            return Err(FromSANError::InvalidMove);
        }

        todo!("Implement the rest of the move_from_san function");

        // let mov = legal_moves
        //     .iter()
        //     .find(|m| {
        //         let source = m.source();
        //         let dest = m.dest();
        //         let moved_piece = board.get_sq_piece(source).unwrap().ptype();

        //         //pawn move starts with the file
        //         match cleaned_san.chars().next().unwrap() {
        //             file @ 'a'..='h' => {
        //                 if moved_piece != PieceType::Pawn {
        //                     return false;
        //                 }

        //                 let file = file as u8 - 'a' as u8;
        //                 if source.file() != file {
        //                     return false;
        //                 }

        //                 //TODO
        //                 return false;
        //             }
        //             //regular move
        //             _ => {
        //                 let piece = PieceType::from(
        //                     cleaned_san.chars().next().unwrap().to_ascii_lowercase(),
        //                 );

        //                 if moved_piece != piece {
        //                     return false;
        //                 }
        //                 let san_dest = Square::try_from(&cleaned_san[cleaned_san.len() - 2..]);

        //                 match san_dest {
        //                     Ok(san_dest) => san_dest == dest,
        //                     Err(_) => false,
        //                 }
        //             }
        //         }

        //         false
        //     })
        //     .ok_or(FromSANError::InvalidMove);

        // mov.cloned()
    }
}

#[derive(Error, Debug)]
enum ToSANError {
    #[error("Move is not a legal move from this position")]
    InvalidMove,
    #[error("Error making move")]
    MoveMakeError,
    #[error("Error undoing move")]
    MoveUndoError,
}

#[derive(Error, Debug)]
enum FromSANError {
    #[error("Move is not a legal move from this position")]
    InvalidMove,
}
