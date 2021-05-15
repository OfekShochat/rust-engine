extern crate chess;
extern crate rand;
use std::time::SystemTime;
use std::collections::HashMap;

#[derive(Clone)]
struct TtEntry {
  eval: i32
}

struct HistoryHeuristics {
  counter_moves: Vec<Vec<chess::ChessMove>>,
}
pub struct Search {
  nodes: i32,
  tt   : HashMap<i32, TtEntry>,
  history
       : HistoryHeuristics
}
impl Search {
  pub fn new() -> Self {
    let mut v: Vec<Vec<chess::ChessMove>> = vec![];
    for _ in 0..64 {
      v.push([].to_vec());
    }
    return Self { nodes: 0, tt: HashMap::new(), history: HistoryHeuristics { counter_moves: v } };
  }

pub fn iterative_deepening(&mut self, board: chess::Board, depth: i8, alpha: i32, beta: i32) -> (i32, chess::ChessMove) {
  let mut result: (i32, chess::ChessMove) = (0, chess::ChessMove::new(chess::Square::A1, chess::Square::A1, None));
  let start = SystemTime::now();
  for d in 1..depth+1 {
    result = self.search(board, d, alpha, beta);
    let duration: u128 = start.elapsed().unwrap().as_millis();
    println!("info depth {} score cp {} nodes {} nps {} time {} pv {}", d, result.0, self.nodes, (self.nodes as f32 /((duration as f32 /1000 as f32)) as f32) as i16, duration, result.1);
  }
  self.nodes = 0;

  return result;
}

fn search(&mut self, board: chess::Board, depth: i8, mut alpha: i32, beta: i32) -> (i32, chess::ChessMove) {
  let mut iterable = chess::MoveGen::new_legal(&board);

  let color = if board.side_to_move() == chess::Color::Black {-1} else {1};  
  let mut best: chess::ChessMove = chess::ChessMove::default();
  for m in &mut iterable {
    let mut result: chess::Board = board.clone();
    board.make_move(m, &mut result);
    let r: i32 = -self.alpha_beta(result, depth - 1, -beta, -alpha, -color);
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

fn quiesce(&mut self, board: chess::Board, mut alpha: i32, beta: i32, color: i8) -> i32 {
  let stand_pat: i32 = self.eval(board) * color as i32;
  if stand_pat >= beta {
    return beta;
  }
  if alpha < stand_pat  {
    alpha = stand_pat;
  }
  let prune_delta: i32 = 900;
  if stand_pat < alpha - prune_delta {
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
    let r: i32 = -self.quiesce(board, -beta, -alpha, -color);
    if r > value {
      value = r;
    }
    if value > alpha {
      alpha = value;
    }
    if r >= beta {
      println!("poop");
      let mut temp: Vec<Vec<chess::ChessMove>> = self.history.counter_moves.clone();
      temp[m.get_source().to_index()][m.get_dest().to_index()] = m.clone();
      self.history.counter_moves = temp;
      return value;
    }
  }
  return value;
}

fn score_counter_moves(&mut self, board: chess::Board) -> Vec<i16> {
  let mut scores: Vec<i16> = vec![];
  let mut iterable = chess::MoveGen::new_legal(&board);
  iterable.set_iterator_mask(!chess::EMPTY);
  for m in &mut iterable {
    let mut result: chess::Board = board.clone();
    board.make_move(m, &mut result);
    if self.history.counter_moves[m.get_source().to_index()].len() > m.get_dest().to_index() {
      println!("{} {}", self.history.counter_moves[m.get_source().to_index()].len(), m.get_dest().to_index());
      if self.history.counter_moves[m.get_source().to_index()][m.get_dest().to_index()] == m {
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
  let mut scores: Vec<i16> = self.score_counter_moves(board);
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

fn alpha_beta(&mut self, board: chess::Board, depth: i8, mut alpha: i32, beta: i32, color: i8) -> i32 {
  let mut iterable = self.order(board);
  if depth == 0 || iterable.len() == 0 {
    /*if board.to_string().contains("r2qk2r/pb4pp/1n2Pb2/2B2Q2/p1p5/2P5/2B2PPP/RN2R1K1 w") && eval(board) == -10000 {
      println!("color: {}", color);
    }*/
    /*if board.side_to_move() == chess::Color::White {
      assert!(color == 1);
    } else {
      assert!(color == -1);
    }*/
    self.nodes += 1;
    return self.quiesce(board, alpha, beta, color);
  }
  let mut value: i32 = -10000;
  for m in &mut iterable {
    self.nodes += 1;
    let r: i32;
    let mut result: chess::Board = board.clone();
    //if self.tt.contains_key(&(result.get_hash() as i32)) {
      //r = self.tt.get(&(result.get_hash() as i32)).get_or_insert_with(|| &TtEntry{ eval: 5 }).eval;
    //} else {
    board.make_move(m, &mut result);
    r = -self.alpha_beta(result, depth - 1, -beta, -alpha, -color);
    //}
    if r > value {
      value = r;
    }
    if value > alpha {
      alpha = value;
      if !self.tt.contains_key(&(result.get_hash() as i32)) {
        self.add_to_tt(result, TtEntry { eval: r });
      }
    }
    if r >= beta {
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

  let eval_board = board.clone();
  // mobility
  let currmobility = chess::MoveGen::new_legal(&eval_board).len() as i32;
  eval_board.null_move();
  let theirmobility = chess::MoveGen::new_legal(&eval_board).len() as i32;
  eval_board.null_move();
  let mobility_score: i32 = currmobility - theirmobility;

  // material
  let mut material: i32 = 0;
  let b: String = board.to_string();
  for i in b.chars() {
    match i {
      'P' => material += 100,
      'R' => material += 500,
      'N' => material += 320,
      'B' => material += 340,
      'Q' => material += 900,

      'p' => material -= 100,
      'r' => material -= 500,
      'n' => material -= 320,
      'b' => material -= 340,
      'q' => material -= 900,
      ' ' => break,
      _   => continue,
    }
  }

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