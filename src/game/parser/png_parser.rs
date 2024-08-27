use std::{collections::HashMap, f32::consts::E};

use thiserror::Error;

use crate::game::{
    board::move_gen::MoveGeneration, castle_rights::CastleType, Board, Color, GameResult, Move,
    PieceType, Square,
};

pub struct PGNParser();

impl PGNParser {
    /// Convert the move to SAN (Standard Algebraic Notation).
    pub fn move_as_san(mov: &Move, board: &mut Board) -> Result<String, ToSANError> {
        let mut move_gen = MoveGeneration::new();
        let legal_moves = move_gen.generate_legal_moves(board);

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
                    m.dest() == dest
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
                }
                if same_file {
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
            let mut move_gen = MoveGeneration::new();
            let legal_response_moves = move_gen.generate_legal_moves(board);
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
    pub fn move_from_san(san: &str, board: &Board) -> Result<Move, FromSANError> {
        if san == "null" {
            return Ok(Move::null());
        }
        //remove unnecessary characters
        let cleaned_san = &san.replace(
            |x| match x {
                'x' | '+' | '#' | '-' => true,
                _ => false,
            },
            "",
        )[..];
        let mut move_gen = MoveGeneration::new();
        let legal_moves = move_gen.generate_legal_moves(board);

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

        if cleaned_san.len() < 2 {
            return Err(FromSANError::InvalidMove);
        }

        //pawn moves or other
        match cleaned_san.chars().next().unwrap() {
            file @ ('a' | 'b' | 'c' | 'd' | 'e' | 'f' | 'g' | 'h') => {
                //handle promotion
                if cleaned_san.contains("=") {
                    let dest = Square::try_from(&cleaned_san[..2])
                        .map_err(|_| FromSANError::ParseError)?;
                    let promotion_piece = PieceType::from(
                        cleaned_san.chars().last().ok_or(FromSANError::ParseError)?,
                    );
                    let mov = legal_moves.iter().find(|m| {
                        m.dest() == dest
                            && file.to_string() == m.source().file_str()
                            && m.flag().is_promotion()
                            && m.flag().promotion_type() == Some(promotion_piece)
                            && board
                                .get_sq_piece_variation(m.source())
                                .is_some_and(|p| p == PieceType::Pawn)
                    });

                    return mov.ok_or(FromSANError::InvalidMove).cloned();
                }

                //multiple pawns could move to the same square
                if cleaned_san.len() == 3 {
                    let dest = Square::try_from(&cleaned_san[1..])
                        .map_err(|_| FromSANError::ParseError)?;
                    let mov = legal_moves.iter().find(|m| {
                        m.dest() == dest
                            && file.to_string() == m.source().file_str()
                            && board
                                .get_sq_piece_variation(m.source())
                                .is_some_and(|p| p == PieceType::Pawn)
                    });

                    return mov.ok_or(FromSANError::InvalidMove).cloned();
                }

                let dest = Square::try_from(cleaned_san).map_err(|_| FromSANError::ParseError)?;
                let mov = legal_moves.iter().find(|m| {
                    m.dest() == dest
                        && board
                            .get_sq_piece_variation(m.source())
                            .is_some_and(|p| p == PieceType::Pawn)
                });

                return mov.ok_or(FromSANError::InvalidMove).cloned();
            }
            _ => {
                let moved_piece = PieceType::from(cleaned_san.chars().next().unwrap());

                //exact move from the moved piece to the destination
                if cleaned_san.len() == 3 {
                    let dest = Square::try_from(&cleaned_san[1..])
                        .map_err(|_| FromSANError::ParseError)?;

                    let mov = legal_moves.iter().find(|m| {
                        m.dest() == dest
                            && board
                                .get_sq_piece_variation(m.source())
                                .is_some_and(|p| p == moved_piece)
                    });

                    return mov.ok_or(FromSANError::InvalidMove).cloned();
                }
                //move with file or rank specified
                if cleaned_san.len() == 4 {
                    let dest = Square::try_from(&cleaned_san[2..])
                        .map_err(|_| FromSANError::ParseError)?;
                    let source = &cleaned_san[1..2];

                    let mov = legal_moves.iter().find(|m| {
                        m.dest() == dest
                            && (source == &m.source().file_str()
                                || source == &m.source().rank_str())
                            && board
                                .get_sq_piece_variation(m.source())
                                .is_some_and(|p| p == moved_piece)
                    });

                    return mov.ok_or(FromSANError::InvalidMove).cloned();
                }
                // move with file and rank specified
                if cleaned_san.len() == 5 {
                    let dest = Square::try_from(&cleaned_san[3..])
                        .map_err(|_| FromSANError::ParseError)?;
                    let source = Square::try_from(&cleaned_san[1..3])
                        .map_err(|_| FromSANError::ParseError)?;

                    let mov = legal_moves.iter().find(|m| {
                        m.dest() == dest
                            && m.source() == source
                            && board
                                .get_sq_piece_variation(m.source())
                                .is_some_and(|p| p == moved_piece)
                    });

                    return mov.ok_or(FromSANError::InvalidMove).cloned();
                }
            }
        };

        Err(FromSANError::InvalidMove)

        // //must be at least 2 characters
        // if cleaned_san.len() < 2 {
        //     return Err(FromSANError::InvalidMove);
        // }

        // todo!("Implement the rest of the move_from_san function");

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

    /// Convert the game to PGN (Portable Game Notation).
    /// The PGN format is described here: https://en.wikipedia.org/wiki/Portable_Game_Notation
    /// Takes a start board, a list of moves that have been played on this board, and an optional hashmap of tags which are added to the pgn string.
    pub fn to_pgn(
        start_board: &Board,
        moves: &[Move],
        tags: Option<&HashMap<String, String>>,
    ) -> Result<String, ToSANError> {
        let mut board = start_board.clone();
        let start_fen = board.to_fen();

        let mut mov_str = String::new();
        for (i, mov) in moves.iter().enumerate() {
            let san = Self::move_as_san(mov, &mut board)?;
            if i % 2 == 0 {
                mov_str.push_str(&format!("{}. {} ", i / 2 + 1, san));
            } else {
                mov_str.push_str(&format!("{} ", san));
            }
        }

        let result = GameResult::get_game_result(&board, None);
        mov_str.push_str(result.into());

        let mut tags_str = String::new();
        if let Some(tags) = tags {
            for (key, value) in tags {
                if key == "result" {
                    continue;
                }
                tags_str.push_str(&format!("[{} \"{}\"]\n", key, value));
            }
        }
        tags_str.push_str(&format!("[result \"{}\"]\n", Into::<&str>::into(result)));
        tags_str.push_str(&format!("[FEN \"{}\"]\n", start_fen));

        Ok(format!("{}\n{}", tags_str, mov_str))
    }

    pub fn from_pgn(pgn: &str) -> Result<PGNData, FromSANError> {
        let (tags, moves) = pgn.split_once("\n\n").ok_or(FromSANError::ParseError)?;

        let mut tags_map = HashMap::new();
        for tag in tags.split('\n') {
            let (key, value) = tag.split_once(" ").ok_or(FromSANError::ParseError)?;
            let key = key.trim();
            let value = value.trim();
            tags_map.insert(
                key[1..].to_string(),
                value[..value.len() - 1].trim_matches('"').to_string(),
            );
        }

        let mut board = tags_map
            .get("FEN")
            .map_or_else(|| Ok(Board::default()), |fen| Board::from_fen(fen))
            .map_err(|_| FromSANError::ParseError)?;

        let mut move_history = Vec::new();
        let moves = moves.trim().split(' ');
        let mut skip_comment = false;
        for mov in moves {
            if skip_comment {
                if mov.ends_with("}") {
                    skip_comment = false;
                }
                continue;
            }

            if mov.starts_with("{") {
                skip_comment = true;
                continue;
            }

            //skip comments and results
            if mov.contains(".") || mov == "*" || mov == "1/2-1/2" || mov == "1-0" || mov == "0-1" {
                continue;
            }

            let mov = Self::move_from_san(mov.trim(), &mut board)?;
            board
                .make_move(&mov, false, true)
                .map_err(|_| FromSANError::InvalidMove)?;
            move_history.push(mov);
        }

        Ok(PGNData {
            tags: tags_map,
            moves: move_history,
            board,
        })
    }
}

pub struct PGNData {
    pub tags: HashMap<String, String>,
    pub moves: Vec<Move>,
    pub board: Board,
}

#[derive(Error, Debug)]
pub enum ToSANError {
    #[error("Move is not a legal move from this position")]
    InvalidMove,
    #[error("Error making move")]
    MoveMakeError,
    #[error("Error undoing move")]
    MoveUndoError,
}

#[derive(Error, Debug)]
pub enum FromSANError {
    #[error("Move is not a legal move from this position")]
    InvalidMove,
    #[error("Error parsing")]
    ParseError,
}
