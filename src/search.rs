use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;
use std::{collections::HashMap, time::Instant};

use chess::{Board, BoardStatus, ChessMove, MoveGen};

use crate::movepick::MovePicker;
use crate::psqt::PSQT;

pub const INF: i32 = 10000;

#[derive(Clone, Copy)]
pub struct TTEntry {
  pub mov: ChessMove,
  score: i32,
  depth: u8,
}

pub struct Manager {
  transpositions: Arc<Mutex<HashMap<u64, TTEntry>>>,
}

impl Manager {
  pub fn new() -> Manager {
    Manager {
      transpositions: Arc::new(Mutex::new(HashMap::with_capacity(1000))),
    }
  }

  pub fn start(&self) {
    self.start_others();
    let t = Arc::clone(&self.transpositions);
    let mut s = SearchWorker::new(t);
    s.iterative_deepening::<true>(chess::Board::default(), -INF, INF, 100);
  }

  fn start_others(&self) {
    for _ in 0..0 {
      let t = Arc::clone(&self.transpositions);
      thread::spawn(move || {
        let mut s = SearchWorker::new(t);
        s.iterative_deepening::<false>(chess::Board::default(), -INF, INF, 100);
      });
    }
  }
}

pub struct SearchWorker {
  pub nodes: usize,
  tt: Arc<Mutex<HashMap<u64, TTEntry>>>,
  best_move: ChessMove,
}

impl SearchWorker {
  pub fn new(tt: Arc<Mutex<HashMap<u64, TTEntry>>>) -> SearchWorker {
    SearchWorker {
      nodes: 0,
      tt,
      best_move: ChessMove::default(),
    }
  }

  pub fn iterative_deepening<const MAIN: bool>(
    &mut self,
    board: Board,
    alpha: i32,
    beta: i32,
    depth: u8,
  ) -> i32 {
    let mut value = 0;
    let start = Instant::now();
    for d in 1..depth {
      let start_depth = Instant::now();
      value = self.search::<true>(board, alpha, beta, d, 1);
      if MAIN {
        println!(
          "info depth {} score cp {} nodes {} nps {} time {} pv {}",
          d,
          value,
          self.nodes,
          (self.nodes as f32 / start_depth.elapsed().as_secs_f32()) as usize,
          start.elapsed().as_millis(),
          self.best_move,
        );
      }
    }
    value
  }

  fn search<const ROOT: bool>(
    &mut self,
    board: Board,
    mut alpha: i32,
    beta: i32,
    depth: u8,
    color: i32,
  ) -> i32 {
    match board.status() {
      BoardStatus::Checkmate => return -INF,
      BoardStatus::Stalemate => return 0,
      _ => {}
    }
    if depth == 0 {
      return self.quiescence(&board, alpha, beta, color);
    }

    let moves = MoveGen::new_legal(&board);
    let mut move_picker = MovePicker::new(moves, self.lock_tt().get(&board.get_hash()));
    let mut best_move = ChessMove::default();
    while let Some(m) = move_picker.next() {
      self.nodes += 1;
      let b = board.make_move_new(m);
      let score = -self.search::<false>(b, -beta, -alpha, depth - 1, -color);
      if score > alpha {
        if ROOT {
          self.best_move = m;
        }
        best_move = m;
        alpha = score
      }
      if score >= beta {
        return beta;
      }
    }

    if best_move != ChessMove::default() {
      self.lock_tt().insert(
        board.get_hash(),
        TTEntry {
          mov: best_move,
          score: alpha,
          depth,
        },
      );
    }
    alpha
  }

  fn quiescence(&mut self, board: &Board, mut alpha: i32, beta: i32, color: i32) -> i32 {
    let stand_pat: i32 = self.evaluate(board) * color;

    if stand_pat >= beta {
      return beta;
    }
    if alpha < stand_pat {
      alpha = stand_pat;
    }

    let mut moves = MoveGen::new_legal(board);
    let captures: &chess::BitBoard = board.color_combined(!board.side_to_move());
    moves.set_iterator_mask(*captures);
    for m in moves {
      self.nodes += 1;
      let b = board.make_move_new(m);
      let score = -self.quiescence(&b, -beta, -alpha, -color);
      if score >= beta {
        return beta;
      } else if score > alpha {
        alpha = score
      }
    }
    alpha
  }

  fn evaluate(&self, board: &Board) -> i32 {
    let mut evaluation = 0;
    for s in chess::ALL_SQUARES {
      let color = board.color_on(s);
      let piece = board.piece_on(s);

      match color {
        Some(chess::Color::White) => evaluation += PSQT[piece.unwrap().to_index()][s.to_index()],
        Some(chess::Color::Black) => {
          evaluation += PSQT[piece.unwrap().to_index() + 6][s.to_index()]
        }
        None => continue,
      }
    }
    evaluation / 512
  }

  fn lock_tt(&mut self) -> MutexGuard<'_, HashMap<u64, TTEntry>> {
    self.tt.lock().unwrap()
  }
}