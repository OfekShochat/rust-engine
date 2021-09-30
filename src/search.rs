use std::sync::{Arc, Mutex};
use std::{collections::HashMap, time::Instant};

use chess::{Board, BoardStatus, MoveGen, Piece};

pub const INF: i32 = 10000;

pub struct TTEntry {
  score: i32
}

pub struct Manager {
  transpositions: Arc<Mutex<HashMap<u64, TTEntry>>>,
}

impl Manager {
  pub fn new() -> Manager {
    Manager { transpositions: Arc::new(Mutex::new(HashMap::new())) }
  }

  pub fn iterative_deepening(&self) {
    let mut s = SearchWorker::new(Arc::clone(&self.transpositions));
    s.iterative_deepening(chess::Board::default(), -INF, INF, 100);
  }
}

pub struct SearchWorker {
  pub nodes: usize,
  tt: Arc<Mutex<HashMap<u64, TTEntry>>>,
}

impl SearchWorker {
  pub fn new(tt: Arc<Mutex<HashMap<u64, TTEntry>>>) -> SearchWorker {
    SearchWorker { nodes: 0, tt }
  }

  pub fn iterative_deepening(&mut self, board: Board, alpha: i32, beta: i32, depth: u8) -> i32 {
    let mut value = 0;
    let start = Instant::now();
    for d in 1..depth {
      let start_depth = Instant::now();
      value = self.search(board, alpha, beta, d, 1);
      println!(
        "info depth {} cp {} nps {} time {}",
        d,
        value,
        self.nodes as f32 / start_depth.elapsed().as_secs_f32(),
        start.elapsed().as_secs_f32()
      );
    }
    value
  }

  fn search(&mut self, board: Board, mut alpha: i32, beta: i32, depth: u8, color: i8) -> i32 {
    match board.status() {
      BoardStatus::Checkmate => return -10000,
      BoardStatus::Stalemate => return 0,
      _ => {}
    }
    if depth == 0 {
      self.nodes += 1;
      return self.quiescence(&board, alpha, beta, color);
    }

    let static_eval = self.evaluate(&board);
    self.tt.lock().unwrap().insert(board.get_hash(), TTEntry { score: static_eval });

    let moves = MoveGen::new_legal(&board);
    for m in moves {
      self.nodes += 1;
      let b = board.make_move_new(m);
      let score = -self.search(b, -beta, -alpha, depth - 1, -color);
      if score > alpha {
        alpha = score
      } else if score >= beta {
        return beta;
      }
    }
    alpha
  }

  fn quiescence(&mut self, board: &Board, mut alpha: i32, beta: i32, color: i8) -> i32 {
    let stand_pat: i32 = self.evaluate(board);
    if stand_pat >= beta {
      return beta;
    }
    if alpha < stand_pat {
      alpha = stand_pat;
    }
    let prune_delta: i32 = 1000;
    if stand_pat < alpha - prune_delta {
      return alpha;
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
        Some(chess::Color::White) => evaluation += self.get_piece_value(piece.unwrap()),
        Some(chess::Color::Black) => evaluation -= self.get_piece_value(piece.unwrap()),
        None => continue,
      }
    }
    evaluation
  }

  fn get_piece_value(&self, piece: Piece) -> i32 {
    match piece {
      Piece::Bishop => 340,
      Piece::Knight => 320,
      Piece::Pawn => 100,
      Piece::Queen => 900,
      Piece::Rook => 500,
      _ => 0,
    }
  }
}
