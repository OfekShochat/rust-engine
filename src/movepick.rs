use chess::{Board, ChessMove, Color, MoveGen, Piece, ALL_PIECES, EMPTY};

use search::get_piece_value;
use search_consts::LOSING_CAPTURE;

use crate::{search::TTEntry, search_consts::INF};

enum GenStage {
  TTMove,
  PrepCaptures,
  Captures,
  PrepQuiets,
  Killers,
  Quites,
}

pub struct MovePicker {
  moves: MoveGen,
  board: Board,
  tt_move: Option<ChessMove>,
  killers: [ChessMove; 2],
  history: [[[i32; 64]; 64]; 2],
  gen_stage: GenStage,
  queue: Vec<(ChessMove, i32)>,
}

impl MovePicker {
  pub fn new(
    board: &Board,
    tt_move: Option<&TTEntry>,
    killers: [ChessMove; 2],
    history: [[[i32; 64]; 64]; 2],
  ) -> MovePicker {
    match tt_move {
      Some(tt_move) => MovePicker {
        moves: MoveGen::new_legal(board),
        board: *board,
        tt_move: Some(tt_move.mov),
        killers,
        history,
        gen_stage: GenStage::TTMove,
        queue: vec![],
      },
      None => MovePicker {
        moves: MoveGen::new_legal(board),
        board: *board,
        tt_move: None,
        killers,
        history,
        gen_stage: GenStage::TTMove,
        queue: vec![],
      },
    }
  }
}

impl Iterator for MovePicker {
  type Item = ChessMove;

  fn next(&mut self) -> Option<ChessMove> {
    match self.gen_stage {
      GenStage::TTMove => {
        self.gen_stage = GenStage::PrepCaptures;
        if let Some(tt_move) = self.tt_move {
          if self.board.legal(tt_move) {
            return self.tt_move;
          } else {
            self.tt_move = None;
          }
        }
        self.next()
      }
      GenStage::PrepCaptures => {
        self.moves.set_iterator_mask(*self.board.combined());
        for m in &mut self.moves {
          if Some(m) != self.tt_move {
            let mut expected_gain = see(self.board, m);
            if expected_gain < 0 {
              expected_gain += LOSING_CAPTURE;
            }
            self.queue.push((m, expected_gain))
          }
        }
        self.gen_stage = GenStage::Captures;
        self.next()
      }
      GenStage::Captures => {
        let mut max = LOSING_CAPTURE;
        let mut best_i = None;
        for (i, &(_, score)) in self.queue.iter().enumerate() {
          if score >= max {
            max = score;
            best_i = Some(i);
          }
        }
        if let Some(best_index) = best_i {
          Some(self.queue.remove(best_index).0)
        } else {
          self.gen_stage = GenStage::PrepQuiets;
          self.next()
        }
      }
      GenStage::PrepQuiets => {
        self.moves.set_iterator_mask(!EMPTY);
        for m in &mut self.moves {
          if Some(m) == self.tt_move {
            continue;
          }
          if let Some(piece) = m.get_promotion() {
            match piece {
              Piece::Queen => self.queue.push((m, INF)),
              _ => self.queue.push((m, -INF)),
            }
            continue;
          }
          let score = match self.board.side_to_move() {
            Color::White => self.history[0][m.get_source().to_index()][m.get_dest().to_index()],
            Color::Black => self.history[1][m.get_source().to_index()][m.get_dest().to_index()],
          };
          self.queue.push((m, score))
        }
        self.gen_stage = GenStage::Killers;
        self.next()
      }
      GenStage::Killers => {
        for m in self.killers {
          if Some(m) != self.tt_move {
            let pos = self.queue.iter().position(|(cm, _)| *cm == m);
            if let Some(pos) = pos {
              self.queue.remove(pos);
              return Some(m);
            }
          }
        }
        self.gen_stage = GenStage::Quites;
        self.next()
      }
      GenStage::Quites => {
        let mut max = 0;
        let mut best_i = None;
        for (i, &(_, score)) in self.queue.iter().enumerate() {
          if best_i.is_none() || score > max {
            max = score;
            best_i = Some(i);
          }
        }
        if let Some(i) = best_i {
          Some(self.queue.remove(i).0)
        } else {
          None
        }
      }
    }
  }
}

fn see(board: Board, mut m: ChessMove) -> i32 {
  let mut index = 0;
  let mut gains = [0; 16];

  let target_square = m.get_dest();
  gains[0] = if let Some(piece) = board.piece_on(target_square) {
    get_piece_value(piece)
  } else {
    0
  };

  'outer: for i in 1..16 {
    let board = board.make_move_new(m);
    gains[i] = get_piece_value(board.piece_on(target_square).unwrap()) - gains[i - 1];
    let color = board.side_to_move();
    let defenders = *board.color_combined(color);
    let blockers = *board.combined();
    let target_square = m.get_dest();
    for p in ALL_PIECES {
      match p {
        Piece::Pawn => {
          let mut potential = chess::get_pawn_attacks(target_square, !color, blockers) &
            defenders &
            board.pieces(Piece::Bishop);
          if potential != EMPTY {
            let attacker = potential.next().unwrap();
            m = ChessMove::new(attacker, target_square, None);
            continue 'outer;
          }
        }
        Piece::Knight => {
          let mut potential =
            chess::get_knight_moves(target_square) & board.pieces(Piece::Knight) & defenders;
          if potential != EMPTY {
            let attacker = potential.next().unwrap();
            m = ChessMove::new(attacker, target_square, None);
            continue 'outer;
          }
        }
        Piece::Bishop => {
          let mut potential = chess::get_bishop_moves(target_square, blockers) &
            defenders &
            board.pieces(Piece::Bishop);
          if potential != EMPTY {
            let attacker = potential.next().unwrap();
            m = ChessMove::new(attacker, target_square, None);
            continue 'outer;
          }
        }
        Piece::Rook => {
          let mut potential =
            chess::get_rook_moves(target_square, blockers) & board.pieces(Piece::Rook) & defenders;
          if potential != EMPTY {
            let attacker = potential.next().unwrap();
            m = ChessMove::new(attacker, target_square, None);
            continue 'outer;
          }
        }
        Piece::Queen => {
          let mut potential = chess::get_rook_moves(target_square, blockers) &
            chess::get_bishop_moves(target_square, blockers) &
            board.pieces(Piece::Queen) &
            defenders;
          if potential != EMPTY {
            let attacker = potential.next().unwrap();
            m = ChessMove::new(attacker, target_square, None);
            continue 'outer;
          }
        }
        Piece::King => {
          let mut potential =
            chess::get_king_moves(target_square) & board.pieces(Piece::King) & defenders;
          if potential != EMPTY {
            let attacker = potential.next().unwrap();
            m = ChessMove::new(attacker, target_square, None);
            continue 'outer;
          }
        }
      }
    }
    index = i;
    break;
  }
  for i in (1..index).rev() {
    gains[i - 1] = -(-gains[i - 1]).max(gains[i])
  }
  gains[0]
}
