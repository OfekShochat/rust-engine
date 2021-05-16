extern crate chess;
use std::time::SystemTime;
use std::collections::HashMap;
pub struct Search {
  nodes: i32,
  tt   : HashMap<i32, TtEntry>,
  history
       : HistoryHeuristics,
  //thread_pool: rayon::ThreadPoolBuilder
  pruned: i32
}

#[derive(Debug, Copy, Clone)]
pub struct Stopper {
  pub st:          SystemTime,
  pub nodes:       i32,
  pub depth:       i16,
  pub time:        i16,
  pub should_stop: bool,
}

struct Psqt {
  wpawn: Vec<i32>,
  wrook: Vec<i32>,
  wknight: Vec<i32>,
  wbishop: Vec<i32>,
  wqueen: Vec<i32>,
  wking: Vec<i32>,

  brook: Vec<i32>,
  bknight: Vec<i32>,
  bbishop: Vec<i32>,
  bqueen: Vec<i32>,
  bking: Vec<i32>,
}

/*
impl Psqt {
  pub fn new() -> Self {
    let Wpawn = (
      0, 0, 0, 0, 0, 0, 0, 0,
      70, 72, 74, 80, 80, 74, 72, 70,
      74, 76, 78, 88, 88, 78, 76, 74,
      75, 80, 82, 90, 90, 82, 80, 75,
      74, 76, 78, 88, 88, 78, 76, 74,
      60, 62, 64, 70, 70, 64, 62, 60,
      60, 62, 64, 70, 70, 64, 62, 60,
      0, 0, 0, 0, 0, 0, 0, 0
    );
  }
}*/

#[derive(Clone)]
struct TtEntry {
  eval: i32
}

struct HistoryHeuristics {
  counter_moves: Vec<Vec<chess::ChessMove>>,
  killers:       Vec<Vec<chess::ChessMove>>
}

impl Search {
  pub fn new() -> Self {
    // setup counter_moves heuristic table
    let mut cm: Vec<Vec<chess::ChessMove>> = vec![];
    let mut ks: Vec<Vec<chess::ChessMove>> = vec![];
    for _ in 0..64 {
      cm.push([chess::ChessMove::new(chess::Square::A1, chess::Square::A1, None); 64].to_vec());
      ks.push([chess::ChessMove::new(chess::Square::A1, chess::Square::A1, None); 64].to_vec());
    }

    return Self { nodes: 0, tt: HashMap::new(), history: HistoryHeuristics { counter_moves: cm, killers: ks }, pruned: 0 };
  }

pub fn iterative_deepening(&mut self, board: chess::Board, alpha: i32, beta: i32, stopper: Stopper) -> (i32, chess::ChessMove) {
  let mut result: (i32, chess::ChessMove) = (0, chess::ChessMove::new(chess::Square::A1, chess::Square::A1, None));
  let start = SystemTime::now();
  for d in 2..stopper.depth+1 {
    result = self.search(board, d as i32, alpha, beta, stopper);
    let duration: u128 = start.elapsed().unwrap().as_millis();
    println!("info depth {} score cp {} nodes {} pruned {} nps {} time {} pv {}", d, result.0, self.nodes, self.pruned, (self.nodes as f32 /((duration as f32 /1000 as f32)) as f32) as i32, duration, result.1);
  }
  self.nodes = 0;

  return result;
}

fn search(&mut self, board: chess::Board, depth: i32, mut alpha: i32, beta: i32, stopper: Stopper) -> (i32, chess::ChessMove) {
  let mut iterable = self.order(board);
  let color = if board.side_to_move() == chess::Color::Black {-1} else {1};  
  let mut best: chess::ChessMove = chess::ChessMove::default();
  for m in &mut iterable {
    if stopper.should_stop {
      break;
    }
    let mut result: chess::Board = board.clone();
    board.make_move(m, &mut result);
    let r: i32 = -self.alpha_beta(result, 0, depth, -beta, -alpha, -color, stopper, 0);
    //println!("{} {}", m, r);
    if r >= beta {
      return (beta, m);
    }
    if r > alpha {
      alpha = r;
      best = m;
    }
  }
  return (alpha, best);
}

fn quiesce(&mut self, board: chess::Board, mut alpha: i32, beta: i32, color: i8, depth: i32) -> i32 {
  let stand_pat: i32 = self.eval(board) * color as i32;
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
    self.nodes += 1;
    let mut result: chess::Board = board.clone();
    board.make_move(m, &mut result);
    let r: i32 = -self.quiesce(board, -beta, -alpha, -color, depth + 1);
    if r > value {
      value = r;
    }
    if value > alpha {
      alpha = value;
    }
    if r >= beta {
      let mut temp: Vec<Vec<chess::ChessMove>> = self.history.counter_moves.clone();
      temp[m.get_source().to_index()][m.get_dest().to_index()] = m.clone();
      self.history.counter_moves = temp;
      return value;
    }
  }
  return value;
}

fn score_killers(&mut self, board: chess::Board) -> Vec<i16> {
  let mut scores: Vec<i16> = vec![];
  let mut iterable = chess::MoveGen::new_legal(&board);
  iterable.set_iterator_mask(!chess::EMPTY);
  for m in &mut iterable {
    let mut result: chess::Board = board.clone();
    board.make_move(m, &mut result);
    if self.history.killers[m.get_source().to_index()].len() > m.get_dest().to_index() {
      if self.history.killers[m.get_source().to_index()][m.get_dest().to_index()] == m {
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

fn order(&mut self, board: chess::Board) -> std::vec::IntoIter<chess::ChessMove> {
  // sum all scores and then order with it.
  let mut scores: Vec<i16> = self.score_killers(board);
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

fn add_to_tt(&mut self, pos: chess::Board, entry: TtEntry) {
  let h: i32 = pos.get_hash() as i32;
  self.tt.insert(h, entry);
}

fn alpha_beta(&mut self, board: chess::Board, curr_depth: i32, max_depth: i32, mut alpha: i32, beta: i32, color: i8, stopper: Stopper, previous_static_eval: i32) -> i32 {
  let mut iterable = self.order(board);
  if curr_depth == max_depth || iterable.len() == 0 {
    self.nodes += 1;
    return self.quiesce(board, alpha, beta, color, 0);
  }
  let eval = self.eval(board);
  let improving: bool = board.checkers().popcnt() == 0 && eval > previous_static_eval;
  if curr_depth < 4 && eval - 225 * curr_depth + 100 * improving as i32 >= beta {
    // Reverse Futility Pruning
    self.pruned += 1;
    return eval;
  }
  let distance_to_leaf = max_depth - curr_depth;
  if curr_depth > 1 && board.checkers().popcnt() == 0 && distance_to_leaf < 4 && eval + 300*distance_to_leaf < beta {
    // Futility Pruning
    self.pruned += 1;
    return eval;
  }
  let mut value: i32 = -10000;
  for m in &mut iterable {
    if stopper.should_stop {
      break;
    }
    self.nodes += 1;
    let r: i32;
    let mut result: chess::Board = board.clone();
    //if self.tt.contains_key(&(result.get_hash() as i32)) {
    //  self.not_searched += 1;
    //  r = self.tt.get(&(result.get_hash() as i32)).unwrap().eval;
    //} else {
    //  self.searched += 1;
    board.make_move(m, &mut result);
    r = -self.alpha_beta(result, curr_depth + 1, max_depth, -beta, -alpha, -color, stopper, eval);
    //}
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
      self.history.killers[m.get_source().to_index()][m.get_dest().to_index()] = m.clone();
      return beta;
    }
  }
  return value;
}

fn eval(&mut self, board: chess::Board) -> i32 {
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
}