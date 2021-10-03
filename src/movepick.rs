use chess::{ChessMove, MoveGen};

use crate::search::TTEntry;

pub struct MovePicker {
  moves: MoveGen,
  tt_move: Option<TTEntry>,
  used_tt_move: bool,
  killers: [ChessMove; 2],
  used_killers: usize,
}

impl MovePicker {
  pub fn new(moves: MoveGen, tt_move: Option<&TTEntry>, killers: [ChessMove; 2]) -> MovePicker {
    match tt_move {
      Some(tte) => MovePicker {
        moves,
        tt_move: Some(*tte),
        used_tt_move: false,
        killers,
        used_killers: 0,
      },
      None => MovePicker {
        moves,
        tt_move: None,
        used_tt_move: false,
        killers,
        used_killers: 0,
      },
    }
  }

  pub fn next(&mut self) -> Option<ChessMove> {
    if !self.used_tt_move && self.tt_move.is_some() {
      let mv = self.tt_move.unwrap().mov;
      self.used_tt_move = true;
      self.moves.remove_move(mv);
      Some(mv)
    } else if self.used_killers < 2 && self.killers[self.used_killers] != ChessMove::default() {
      Some(self.killers[self.used_killers])
    } else {
      self.moves.next()
    }
  }
}
