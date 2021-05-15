mod search;
extern crate chess;
use std::str::FromStr;
use std::io::stdin;

#[allow(unused_must_use)]
fn main() {
  let mut searcher = search::Search::new();
  loop {
    let mut s:String = String::new();
    stdin().read_line(&mut s);
    let board: chess::Board = chess::Board::from_str(&s.to_string()).unwrap();
    let result: (i32, chess::ChessMove) = searcher.iterative_deepening(board, 100, -10000, 10000);
    println!("bestmove {}", result.1);
  }
}