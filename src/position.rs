use chess::Board;
use std::str::FromStr;

pub struct Position {
  boards: Vec<Board>
}

impl Position {
  pub fn new(fen: &str) -> Position {
    Position { boards: vec![Board::from_str(fen).unwrap()] }
  }

  pub fn from_board(board: Board) -> Position {
    Position { boards: vec![board] }
  }

  #[inline]
  pub fn in_check(&self) -> bool {
    self.board().checkers().popcnt() != 0
  }

  #[inline]
  pub fn is_draw(&self) -> bool {
    let hash = self.hash();
    self.boards.iter().rev().skip(1).filter(|board| board.get_hash() == hash).count() >= 2 || self.board().combined().popcnt() == 2
  }

  #[inline]
  pub fn board(&self) -> &Board {
    self.boards.last().unwrap()
  }

  #[inline]
  pub fn hash(&self) -> u64 {
    self.board().get_hash()
  }
}