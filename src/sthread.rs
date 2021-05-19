use std::sync::{mpsc, Arc};
use std::sync::atomic::{AtomicBool, AtomicI32};
use std::{time::SystemTime, vec};
use std::collections::HashMap;
extern crate chess;

static MAX_DEPTH: i16 = 100;

#[derive(Debug, Copy, Clone)]
pub struct Stopper {
  pub st:          SystemTime,
  pub nodes:       i32,
  pub depth:       i16,
  pub time:        i16,
  pub should_stop: bool,
}

#[derive(Clone)]
pub struct ThreadMessage {
  // type of message
  get_status: bool,
  mission:    bool,

}

#[derive(Clone)]
pub struct TtEntry {
  eval: i32
}

#[derive(Clone)]
pub struct HistoryHeuristics {
  pub counter_moves: Vec<Vec<i32>>,
  pub killers:       Vec<Vec<chess::ChessMove>>
}

#[derive(Clone)]
pub struct ThreadManager {
  id: usize,
  nodes: Arc<AtomicI32>,
  should_stop: Arc<AtomicBool>,
  sender: mpsc::Sender<ThreadMessage>,
  score: i32,
  history
       : HistoryHeuristics,
}

pub fn iterative_deepening(tm: *mut ThreadManager, board: chess::Board, alpha: i32, beta: i32) {
  let color = if board.side_to_move() == chess::Color::Black {-1} else {1};
  for d in 2..MAX_DEPTH {
    unsafe{(*tm).score = alpha_beta(tm, board, 0, d as i32, alpha, beta, color, 0)};
  }
}

fn alpha_beta(mut tm: *mut ThreadManager, board: chess::Board, curr_depth: i32, max_depth: i32, mut alpha: i32, beta: i32, color: i8, previous_static_eval: i32) -> i32 {
  let mut iterable = order(tm, board);
  if curr_depth == max_depth || iterable.len() == 0 {
    return quiesce(board, alpha, beta, color, 0);
  }
  let eval = eval(board) * color as i32;
  let improving: bool = board.checkers().popcnt() == 0 && eval > previous_static_eval;
  if curr_depth < 4 && eval - 225 * curr_depth + 100 * improving as i32 >= beta {
    // Reverse Futility Pruning
    return eval;
  }
  let distance_to_leaf = max_depth - curr_depth;
  if curr_depth > 1 && board.checkers().popcnt() == 0 && distance_to_leaf < 4 && eval + 300*distance_to_leaf < beta {
    // Futility Pruning
    return eval;
  }
  let mut value: i32 = -10000;
  for m in &mut iterable {
    let r: i32;
    let mut result: chess::Board = board.clone();
    //if self.tt.contains_key(&(result.get_hash() as i32)) {
    //  self.not_searched += 1;
    //  r = self.tt.get(&(result.get_hash() as i32)).unwrap().eval;
    //} else {
    //  self.searched += 1;
    board.make_move(m, &mut result);
    r = -alpha_beta(tm, result, curr_depth + 1, max_depth, -beta, -alpha, -color, eval);
    //}
    if curr_depth == 0 {
      unsafe{(*tm).history.counter_moves[m.get_source().to_index()][m.get_dest().to_index()] = r};
    }
    if r > value {
      value = r;
    }
    if value > alpha {
      alpha = value;
      /*if !self.tt.contains_key(&(result.get_hash() as i32)) {
        self.add_to_tt(result, TtEntry { eval: r });
      }*/
    }
    if r >= beta {
      unsafe{(*tm).history.killers[m.get_source().to_index()][m.get_dest().to_index()] = m.clone()};
      return beta;
    }
  }
  return value;
}

fn quiesce(board: chess::Board, mut alpha: i32, beta: i32, color: i8, depth: i32) -> i32 {
  let stand_pat: i32 = eval(board) * color as i32;
  if stand_pat >= beta {
    return beta;
  }
  if alpha < stand_pat  {
    alpha = stand_pat;
  }
  let prune_delta: i32 = 1000;
  if stand_pat < alpha - prune_delta {
    return alpha;
  }

  if depth > 4 {
    return alpha;
  }

  let mut value: i32 = -10000;
  let mut iterable: chess::MoveGen = chess::MoveGen::new_legal(&board);
  let captures: &chess::BitBoard = board.color_combined(!board.side_to_move());
  iterable.set_iterator_mask(*captures);
  for m in &mut iterable {
    let mut result: chess::Board = board.clone();
    board.make_move(m, &mut result);
    let r: i32 = -quiesce(board, -beta, -alpha, -color, depth + 1);
    if r > value {
      value = r;
    }
    if value > alpha {
      alpha = value;
    }
    if r >= beta {
      return value;
    }
  }
  return value;
}

fn score_killers(tm: *mut ThreadManager, board: chess::Board) -> Vec<i32> {
  let mut scores: Vec<i32> = vec![];
  let mut iterable = chess::MoveGen::new_legal(&board);
  iterable.set_iterator_mask(!chess::EMPTY);
  for m in &mut iterable {
    let mut result: chess::Board = board.clone();
    board.make_move(m, &mut result);
    if unsafe{(*tm).history.killers[m.get_source().to_index()].len() > m.get_dest().to_index()} {
      if unsafe{(*tm).history.killers[m.get_source().to_index()][m.get_dest().to_index()] == m} {
        scores.push(10);
      } else {
        scores.push(0);
      }
    } else {
      scores.push(0);
    }
  }
  return scores;
}

fn score_counters(tm: *mut ThreadManager, board: chess::Board) -> Vec<i32> {
  let mut iterable = chess::MoveGen::new_legal(&board);
  iterable.set_iterator_mask(!chess::EMPTY);
  let mut scores = vec![];
  for m in iterable {
    scores.push(unsafe{(*tm).history.counter_moves[m.get_source().to_index()][m.get_dest().to_index()]});
  }
  return scores;
}

fn order(tm: *mut ThreadManager, board: chess::Board) -> std::vec::IntoIter<chess::ChessMove> {
  // sum all scores and then order with it.
  let mut scores = vec![];
  let ks = score_killers(tm, board);
  let cs = score_counters(tm, board);
  for i in 0..cs.len() {
    scores.push(cs[i] + ks[i]);
  }
  let mut iterable = chess::MoveGen::new_legal(&board);
  iterable.set_iterator_mask(!chess::EMPTY);
  let mut moves: Vec<chess::ChessMove> = [].to_vec();
  for i in iterable {
    moves.push(i)
  }

  for i in 0..scores.len() {
    if let Some((j, _)) = scores.iter()
                              .enumerate()
                              .skip(i)
                              .min_by_key(|x| x.1) {
      scores.swap(i, j);
      moves.swap(i, j);
    }
  }
  return moves.into_iter();
}


fn eval(board: chess::Board) -> i32 {
  let s: chess::BoardStatus = board.status();
  if !(s == chess::BoardStatus::Ongoing) {
    if s == chess::BoardStatus::Checkmate {
      return -10000;
    }
    return 0;
  }

  // material
  let mut material: i32 = 0;
  let b: String = board.to_string();
  let mut count = 0;
  for i in b.chars() {
    count += 1;
    match i {
      'P' => material += 100 * ((if count <= 4 {count / 4} else {4/count}) as i32).abs(),
      'R' => material += 500,
      'N' => material += 320,
      'B' => material += 340,
      'Q' => material += 900,

      'p' => material -= 100 * ((if count <= 3 {count / 3} else {3/count}) as i32).abs(),
      'r' => material -= 500,
      'n' => material -= 320,
      'b' => material -= 340,
      'q' => material -= 900,
      '/' => count += 1,
      ' ' => break,
      _   => continue,
    }
  }

  if material > 600 || material < -600 {
    return material;
  }

  let eval_board = board.clone();
  // mobility
  let currmobility = chess::MoveGen::new_legal(&eval_board).len() as i32;
  eval_board.null_move();
  let theirmobility = chess::MoveGen::new_legal(&eval_board).len() as i32;
  eval_board.null_move();
  let mobility_score: i32 = currmobility - theirmobility;

  // castling
  let mut castling_score: i32 = 0;
  if board.castle_rights(chess::Color::White) == chess::CastleRights::Both {
    castling_score += 10;
  } else if board.castle_rights(chess::Color::White) == chess::CastleRights::KingSide || board.castle_rights(chess::Color::White) == chess::CastleRights::QueenSide {
    castling_score += 5;
  }

  if board.castle_rights(chess::Color::Black) == chess::CastleRights::Both {
    castling_score -= 10;
  } else if board.castle_rights(chess::Color::Black) == chess::CastleRights::KingSide || board.castle_rights(chess::Color::Black) == chess::CastleRights::QueenSide {
    castling_score -= 5;
  }
  return material + mobility_score + castling_score;
}