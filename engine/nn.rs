use chess::{Board, Color, ChessMove};
use packed_simd::f32x4;

use net::*;

fn dot(a: &[f32], b: &[f32]) -> f32 {
  assert_eq!(a.len(), b.len());
  assert!(a.len() % 4 == 0);

  a.chunks_exact(4)
    .map(f32x4::from_slice_unaligned)
    .zip(b.chunks_exact(4).map(f32x4::from_slice_unaligned))
    .map(|(a, b)| a * b)
    .sum::<f32x4>()
    .sum()
}

pub struct Net {
  w1: [[f32; 768]; 128],
  w2: [[f32; 128]; 1],
  b1: [f32; 128],
  b2: [f32; 1],
  accumulator: [f32; 128],
  board_rep: [f32; 768],
  features: Vec<(usize, usize)>
}

impl Net {
  pub fn new(w1: [[f32; 768]; 128], w2: [[f32; 128]; 1], b1: [f32; 128], b2: [f32; 1]) -> Net {
    Net {
      w1,
      w2,
      b1,
      b2,
      accumulator: [0.0; 128],
      board_rep: [0.0; 768],
      features: vec![]
    }
  }

  pub fn from_file() -> Net {
    Net::new(FC0_WEIGHT, FC1_WEIGHT, FC0_BIAS, FC1_BIAS)
  }

  pub fn apply_move(&mut self, board: &Board, mov: ChessMove) {
    let piece = board.piece_on(mov.get_source());
    match board.side_to_move() {
      Color::White => {
        self.features.push((piece.unwrap().to_index() * 64 + mov.get_dest().to_index(), piece.unwrap().to_index() * 64 + mov.get_source().to_index()))
      }
      Color::Black => {
        self.features.push(((piece.unwrap().to_index() + 6) * 64 + mov.get_dest().to_index(), (piece.unwrap().to_index() + 6) * 64 + mov.get_source().to_index()))
      }
    }
  }

  pub fn pop_move(&mut self) {
    self.features.pop();
  }

  pub fn eval(&mut self, board: &Board) -> i32 {
    let mut inputs = [0.0; 768];
    match board.side_to_move() {
      Color::White => {
        for s in chess::ALL_SQUARES {
          let color = board.color_on(s);
          let piece = board.piece_on(s);

          match color {
            Some(chess::Color::White) => inputs[piece.unwrap().to_index() * 64 + s.to_index()] = 1.0,
            Some(chess::Color::Black) => inputs[(piece.unwrap().to_index() + 6) * 64 + s.to_index()] = 1.0,
            None => continue,
          }
        }
      }
      Color::Black => {
        for s in chess::ALL_SQUARES {
          let color = board.color_on(s);
          let piece = board.piece_on(s);

          match color {
            Some(chess::Color::White) => inputs[(piece.unwrap().to_index() + 6) * 64 + s.to_index()] = 1.0,
            Some(chess::Color::Black) => inputs[piece.unwrap().to_index() * 64 + s.to_index()] = 1.0,
            None => continue,
          }
        }
      }
    }

    self.forward(inputs)
  }

  fn forward(&mut self, inputs: [f32; 768]) -> i32 {
    let mut b = self.b1;
    if self.accumulator == [0.0; 128] {
      self.features.pop(); // first move doesnt count.
      for w in 0..self.w1.len() {
        b[w] += dot(&inputs, &self.w1[w]);
      }
      self.accumulator = b;
      self.board_rep = inputs;
    } else {
      b = self.accumulator;
      for (added, removed) in &self.features {
        for i in self.w1 {
          for d in 0..b.len() {
            b[d] += i[*added];
          }
          for d in 0..b.len() {
            b[d] -= i[*removed];
          }
        }
      }
    }
    self.relu(&mut b);

    let mut c = self.b2;
    for w in 0..self.w2.len() {
      c[w] += dot(&b, &self.w2[w]);
    }

    unsafe { (*c.get_unchecked(0) * 400.0) as i32 }
  }

  fn relu(&self, a: &mut [f32]) {
    for i in 0..a.len() {
      if a[i] > 0.0 {
        a[i] = 0.0
      }
    }
  }
}
