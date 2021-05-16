extern crate chess;
use std::str::FromStr;
mod search;
use std::time::SystemTime;

pub struct UciParser {
  alpha: i32,
  beta : i32,
  searcher: search::Search,
  executer: UciFunctions
}

impl UciParser {
  pub fn new() -> Self {
    return Self { alpha: 10000, beta: -10000, searcher: search::Search::new(), executer: UciFunctions::new() }
  }
}

pub struct UciFunctions {
  searcher: search::Search,
  board:    chess::Board
}

impl UciFunctions {
  pub fn new() -> Self {
    return Self { searcher: search::Search::new(), board: chess::Board::default() }
  }

  pub fn position(&mut self, fen: String, startpos: bool, moves: String) {
    if fen.len() != 0 {
      self.board = chess::Board::from_str(&fen).unwrap();
    } else if startpos {
      self.board = chess::Board::default();
    } if moves.len() != 0 {
      let splitted = moves.split(" ");
      for i in splitted {
        let mut temp: chess::Board = self.board.clone();
        self.board.make_move(chess::ChessMove::from_str(i).unwrap(), &mut temp);
        self.board = temp;
      }
    }
  }

  pub fn go(&mut self, depth: i16, nodes: i32, time: i16, timemn: bool) {
    let start = SystemTime::now();
    let mut stopper: search::Stopper = search::Stopper { st: start, nodes: nodes, depth: depth, time: time, should_stop: false };
    self.searcher.iterative_deepening(self.board, -10000, 10000, stopper);
  }
}