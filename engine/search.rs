use std::str::FromStr;
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;
use std::{collections::HashMap, time::Instant};

use chess::{Board, BoardStatus, ChessMove, Color, MoveGen, Piece};

use crate::movepick::MovePicker;
use crate::nn::Net;
use crate::psqt::PSQT;

const INF: i32 = 10000;
const MAX_PLY: u8 = 100;

#[derive(Clone, Copy)]
pub struct TTEntry {
  pub mov: ChessMove,
  score: i32,
  depth: u8,
}

#[derive(Clone, Copy)]
pub struct Limit {
  time: u128,
  depth: u8,
  started: Instant,
}

impl Limit {
  pub fn timed(time: u128) -> Limit {
    Limit {
      time,
      depth: MAX_PLY,
      started: Instant::now(),
    }
  }

  pub fn depthed(depth: u8) -> Limit {
    Limit {
      time: 3600000,
      depth,
      started: Instant::now(),
    }
  }

  pub fn check(&mut self, depth: u8) -> bool {
    if self.started.elapsed().as_millis() > self.time || self.depth <= depth {
      true
    } else {
      false
    }
  }
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

  pub fn start(&self, pos: String, lim: Limit) {
    self.start_others(pos.clone(), lim);
    let tt = Arc::clone(&self.transpositions);
    let mut s = SearchWorker::new(tt, lim);
    s.iterative_deepening::<true>(
      chess::Board::from_str(pos.as_str()).unwrap(),
      -INF,
      INF,
      MAX_PLY,
    );
  }

  fn start_others(&self, pos: String, lim: Limit) {
    for _ in 0..0 {
      let tt = Arc::clone(&self.transpositions);
      let pos = pos.clone();
      thread::spawn(move || {
        let mut s = SearchWorker::new(tt, lim);
        s.iterative_deepening::<false>(
          chess::Board::from_str(pos.as_str()).unwrap(),
          -INF,
          INF,
          MAX_PLY,
        );
      });
    }
  }
}

pub struct SearchWorker {
  nodes: usize,
  seld_depth: usize,
  tt: Arc<Mutex<HashMap<u64, TTEntry>>>,
  killers: [[ChessMove; 2]; MAX_PLY as usize],
  best_move: ChessMove,
  temp: ChessMove,
  lim: Limit,
  net: Net,
}

impl SearchWorker {
  pub fn new(tt: Arc<Mutex<HashMap<u64, TTEntry>>>, lim: Limit) -> SearchWorker {
    SearchWorker {
      nodes: 0,
      seld_depth: 0,
      tt,
      killers: [[ChessMove::default(); 2]; MAX_PLY as usize],
      best_move: ChessMove::default(),
      temp: ChessMove::default(),
      lim,
      net: Net::from_file(),
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
      value = self.search::<true, false>(board, alpha, beta, d, 0);
      self.best_move = self.temp;
      if MAIN {
        println!(
          "info depth {} seldepth {} score cp {} nodes {} nps {} time {} pv {}",
          d,
          self.seld_depth,
          value,
          self.nodes,
          (self.nodes as f32 / start_depth.elapsed().as_secs_f32()) as usize,
          start.elapsed().as_millis(),
          self.best_move,
        );
      }
      if self.lim.check(d.into()) {
        break;
      }
    }
    if MAIN {
      println!("bestmove {}", self.best_move);
    }
    value
  }

  fn search<const ROOT: bool, const IN_NULL: bool>(
    &mut self,
    board: Board,
    mut alpha: i32,
    beta: i32,
    depth: u8,
    curr_depth: i32,
  ) -> i32 {
    match board.status() {
      BoardStatus::Checkmate => return -INF + curr_depth,
      BoardStatus::Stalemate => return 0,
      _ => {}
    }
    if board.combined().popcnt() < 3 {
      return 0; // KvK
    }

    if depth <= 0 {
      return self.quiescence(&board, alpha, beta, curr_depth);
    }

    if board.combined().popcnt() > 5 && board.checkers().popcnt() == 0 && !IN_NULL && depth >= 4 {
      let b = board.null_move().unwrap();
      let r = self.search::<false, true>(b, -beta, -beta + 1, depth - 4, curr_depth + 1);
      if r >= beta {
        return beta;
      }
    }

    if depth == 8 {
      let bound = (15 + beta) / 10;
      if self.search::<false, false>(board, bound - 1, bound, 4, curr_depth + 1) >= bound {
        return beta;
      }

      let bound = (-15 + alpha) / 10;
      if self.search::<false, false>(board, bound, bound + 1, 4, curr_depth + 1) <= bound {
        return alpha;
      }
    }

    let static_eval = self.evaluate(&board);
    if curr_depth < 7 && static_eval - 175 * curr_depth / 2 >= beta {
      return static_eval;
    }

    let mut reductions = 0;
    let mut range_strength: u8 = 0;

    let moves = MoveGen::new_legal(&board);
    let mut killers = self.killers[curr_depth as usize];
    let mut move_picker = MovePicker::new(moves, self.lock_tt().get(&board.get_hash()), killers);
    let mut best_move = ChessMove::default();
    while let Some(m) = move_picker.next() {
      self.nodes += 1;
      let b = board.make_move_new(m);
      let score = -self.search::<false, false>(
        b,
        -beta,
        -alpha,
        depth -
          1 -
          if depth > reductions {
            reductions as u8
          } else {
            depth - 1
          },
        curr_depth + 1,
      );

      if range_strength < 3 && static_eval - score < 30 {
        range_strength += 1;
        if range_strength > 2 {
          reductions += 1
        }
      }

      if score > alpha {
        if ROOT {
          self.temp = m;
        }
        best_move = m;
        alpha = score
      }
      if score >= beta {
        if board.color_on(m.get_dest()).is_none() {
          killers[0] = killers[1];
          killers[0] = m;
        }
        return beta;
      }

      if self.nodes % 1024 == 0 && self.lim.check(curr_depth as u8) {
        return self.evaluate(&board);
      }
    }

    self.killers[curr_depth as usize] = killers;

    if best_move != ChessMove::default() {
      self.lock_tt().insert(
        board.get_hash(),
        TTEntry {
          mov: best_move,
          score: alpha,
          depth: curr_depth as u8,
        },
      );
    }
    alpha
  }

  fn quiescence(&mut self, board: &Board, mut alpha: i32, beta: i32, curr_depth: i32) -> i32 {
    self.seld_depth = self.seld_depth.max(curr_depth as usize);
    let mut stand_pat = self.evaluate(board);
    if -200 < stand_pat && stand_pat < 200 {
      stand_pat = self.net.eval(board);
    }

    if stand_pat >= beta {
      return beta;
    }

    if stand_pat + 975 < alpha {
      return alpha;
    }

    if alpha < stand_pat {
      alpha = stand_pat;
    }

    let mut moves = MoveGen::new_legal(board);
    let captures: &chess::BitBoard = board.color_combined(!board.side_to_move());
    moves.set_iterator_mask(*captures);
    for m in moves {
      let futility = stand_pat + 40;
      let piece_value = self.get_piece_value(board.piece_on(m.get_dest()).unwrap());
      if piece_value + futility <= alpha && board.piece_on(m.get_source()).unwrap() != Piece::Pawn {
        continue;
      }

      self.nodes += 1;
      let b = board.make_move_new(m);
      let score = -self.quiescence(&b, -beta, -alpha, curr_depth + 1);
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
    evaluation / 512 *
      if board.side_to_move() == Color::Black {
        -1
      } else {
        1
      }
  }

  fn get_piece_value(&self, piece: Piece) -> i32 {
    match piece {
      Piece::Bishop => 340,
      Piece::Knight => 320,
      Piece::Pawn => 100,
      Piece::Queen => 900,
      Piece::Rook => 500,
      _ => unreachable!(),
    }
  }

  fn lock_tt(&mut self) -> MutexGuard<'_, HashMap<u64, TTEntry>> {
    self.tt.lock().unwrap()
  }
}