mod search;
extern crate chess;
use std::str::FromStr;

fn main() {
  let board = chess::Board::from_str("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
  let result: (i16, chess::ChessMove) = search::iterative_deepening(board, 5, -10000, 5000);
  println!("eval: {}, move: {}", result.0, result.1);
}