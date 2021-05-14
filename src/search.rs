extern crate chess;
extern crate rand;
use std::time::SystemTime;

pub fn iterative_deepening(board: chess::Board, depth: i8, alpha: i16, beta: i16) -> (i16, chess::ChessMove) {
  let mut result: (i16, chess::ChessMove) = (0, chess::ChessMove::new(chess::Square::A1, chess::Square::A1, None));
  for d in 1..depth+1 {
    let start = SystemTime::now();
    result = search(board, d, alpha, beta);
    let duration: u128 = start.elapsed().unwrap().as_millis();
    println!("info depth {} score cp {} nodes {} nps {} time {} pv {}", d, result.0, 1, 1, duration, result.1);
  }

  return result;
}

pub fn search(board: chess::Board, depth: i8, mut alpha: i16, beta: i16) -> (i16, chess::ChessMove) {
  let mut iterable = chess::MoveGen::new_legal(&board);
  //let targets: &chess::BitBoard = board.color_combined(!board.side_to_move());
  let color = if board.side_to_move() == chess::Color::Black {-1} else {1};  
  let mut best: chess::ChessMove = chess::ChessMove::default();
  iterable.set_iterator_mask(!chess::EMPTY);
  for m in &mut iterable {
    let mut result: chess::Board = board.clone();
    board.make_move(m, &mut result);
    let r: i16 = -alpha_beta(result, depth - 1, alpha, beta, -color);
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

fn alpha_beta(board: chess::Board, depth: i8, mut alpha: i16, beta: i16, color: i8) -> i16 {
  let mut iterable = chess::MoveGen::new_legal(&board);
  if depth == 0 || iterable.len() == 0 {
    /*if board.to_string().contains("r2qk2r/pb4pp/1n2Pb2/2B2Q2/p1p5/2P5/2B2PPP/RN2R1K1 w") && eval(board) == -10000 {
      println!("color: {}", color);
    }*/
    if board.side_to_move() == chess::Color::White {
      assert!(color == 1);
    } else {
      assert!(color == -1)
    }
    return eval(board) * color as i16;
  }
  
  //let targets: &chess::BitBoard = board.color_combined(!board.side_to_move());
  iterable.set_iterator_mask(!chess::EMPTY);
  for m in &mut iterable {
    let mut result: chess::Board = board.clone();
    board.make_move(m, &mut result);
    let r = -alpha_beta(result, depth - 1, alpha, beta, -color);
    if r >= beta {
      return beta;
    }
    if r > alpha {
      alpha = r;
    }
  }
  return alpha;
}

fn eval(board: chess::Board) -> i16 {
  let s: chess::BoardStatus = board.status();
  if !(s == chess::BoardStatus::Ongoing) {
    if s == chess::BoardStatus::Checkmate {
      return -10000;
    }
    if s == chess::BoardStatus::Stalemate {
      return 0;
    }
  }

  let eval_board = board.clone();
  // mobility
  let currmobility = chess::MoveGen::new_legal(&eval_board).len() as i16;
  eval_board.null_move();
  let theirmobility = chess::MoveGen::new_legal(&eval_board).len() as i16;
  eval_board.null_move();
  let mobility_score: i16 = currmobility - theirmobility;

  // material
  let mut material: i16 = 0;
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
  let mut castling_score: i16 = 0;
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

/*
fn roll_out(board: chess::Board) -> f32 {
  let mut value: f32 = 0.0;
  println!("{}", board.to_string());
  for _ in 1..5 {
    println!("poop");
    let mut result: chess::Board = board.clone();
    while result.status() == chess::BoardStatus::Ongoing {
      let mut iterable = chess::MoveGen::new_legal(&result);
      iterable.set_iterator_mask(!chess::EMPTY);
      let mut rng = rand::thread_rng();
      //println!("{} {}", rng.gen_range(0..iterable.len()), iterable.len());
      board.make_move(iterable.nth(rng.gen_range(0..iterable.len())).unwrap(), &mut result);
      //println!("{}", result.to_string());
    }
    println!("{}", value);
    if board.status() == chess::BoardStatus::Checkmate {
      if board.side_to_move() == chess::Color::Black {
        value += 1.0;
      } else {
        value -= 1.0;
      }
    }
    else if board.status() == chess::BoardStatus::Stalemate {
      value += 0.0
    }
  }
  return value;
}
*/