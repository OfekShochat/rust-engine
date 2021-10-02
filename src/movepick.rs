use chess::{ChessMove, MoveGen};

use crate::search::TTEntry;

pub struct MovePicker {
  moves: MoveGen,
  tt_move: Option<TTEntry>,
  used_tt_move: bool,
}

impl MovePicker {
  pub fn new(moves: MoveGen, tt_move: Option<&TTEntry>) -> MovePicker {
    match tt_move {
      Some(tte) => {
        MovePicker { moves, tt_move: Some(*tte), used_tt_move: false }
      }
      None => MovePicker { moves, tt_move: None, used_tt_move: false }
    }
  }

  pub fn next(&mut self) -> Option<ChessMove> {
    if !self.used_tt_move && self.tt_move.is_some() {
      let mv = self.tt_move.unwrap().mov;
      self.used_tt_move = true;
      self.moves.remove_move(mv);
      return Some(mv);
    }
    self.moves.next()
  }
}