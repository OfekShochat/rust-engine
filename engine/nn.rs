use chess::{Board, Color, BitBoard, EMPTY, Piece};
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

#[inline]
fn relu(a: &mut [f32]) {
  for i in 0..a.len() {
    if a[i] < 0.0 {
      a[i] = 0.0;
    }
  }
}

pub struct Nnue {
  incremental: IncrementalLayer<768, 128>,
  fc1: Layer<128, 1>,
}

impl Nnue {
  pub fn new() -> Nnue {
    Nnue {
      incremental: IncrementalLayer::new(FC0_WEIGHT, FC0_BIAS),
      fc1: Layer::new(FC1_WEIGHT, FC1_BIAS),
    }
  }

  pub fn eval(&mut self, board: &Board) -> i32 {
    let io = self.incremental.forward(board);
    let o = self.fc1.forward(io);
    (o[0] * 200.0) as i32
  }
}

struct IncrementalLayer<const INPUTS: usize, const OUTPUTS: usize> {
  w: [[f32; OUTPUTS]; INPUTS],
  accumulator: [f32; OUTPUTS],
  white: BitBoard,
  black: BitBoard,
  pawns: BitBoard,
  knights: BitBoard,
  bishops: BitBoard,
  rooks: BitBoard,
  queens: BitBoard,
  kings: BitBoard,
  input: [[f32; 64]; 12],
}

impl <const INPUTS: usize, const OUTPUTS: usize> IncrementalLayer<INPUTS, OUTPUTS> {
  pub fn new(w: [[f32; INPUTS]; OUTPUTS], b: [f32; OUTPUTS]) -> IncrementalLayer<INPUTS, OUTPUTS> {
    IncrementalLayer { 
      w: IncrementalLayer::prepare(w),
      accumulator: b,
      white: EMPTY,
      black: EMPTY,
      pawns: EMPTY,
      knights: EMPTY,
      bishops: EMPTY,
      rooks: EMPTY,
      queens: EMPTY,
      kings: EMPTY,
      input: [[0.0; 64]; 12],
    }
  }

  fn prepare(weights: [[f32; INPUTS]; OUTPUTS]) -> [[f32; OUTPUTS]; INPUTS]  {
    let mut out = [[0.0; OUTPUTS]; INPUTS];
    for w in 0..weights.len() {
      for j in 0..weights[w].len() {
        out[j][w] = weights[w][j]
      }
    }
    out
  }

  pub fn forward(&mut self, board: &Board) -> [f32; OUTPUTS] {
    // full credit to blackmarlin engine https://github.com/dsekercioglu/blackmarlin/blob/main/src/bm/nnue.rs
    let white = *board.color_combined(Color::White);
    let black = *board.color_combined(Color::Black);

    let pawns = *board.pieces(Piece::Pawn);
    let knights = *board.pieces(Piece::Knight);
    let bishops = *board.pieces(Piece::Bishop);
    let rooks = *board.pieces(Piece::Rook);
    let queens = *board.pieces(Piece::Queen);
    let kings = *board.pieces(Piece::King);

    let changes = [
      (white & pawns) ^ (self.white & self.pawns),
      (white & knights) ^ (self.white & self.knights),
      (white & bishops) ^ (self.white & self.bishops),
      (white & rooks) ^ (self.white & self.rooks),
      (white & queens) ^ (self.white & self.queens),
      (white & kings) ^ (self.white & self.kings),
      (black & pawns) ^ (self.black & self.pawns),
      (black & knights) ^ (self.black & self.knights),
      (black & bishops) ^ (self.black & self.bishops),
      (black & rooks) ^ (self.black & self.rooks),
      (black & queens) ^ (self.black & self.queens),
      (black & kings) ^ (self.black & self.kings),
    ];

    self.white = white;
    self.black = black;
    self.pawns = pawns;
    self.knights = knights;
    self.bishops = bishops;
    self.rooks = rooks;
    self.queens = queens;
    self.kings = kings;

    for (index, (p, &bb)) in self.input.iter_mut().zip(&changes).enumerate() {
      for sq in bb {
        let input = &mut p[sq.to_index()];
        let old = *input;
        let new = 1.0 - old;
        *input = new;

        if new == 1.0 {
          for (acc, w) in self.accumulator.iter_mut().zip(self.w[index * 64 + sq.to_index()]) {
            *acc += w;
          }
        } else {
          for (acc, w) in self.accumulator.iter_mut().zip(self.w[index * 64 + sq.to_index()]) {
            *acc -= w;
          }
        }
      }
    }
    let mut out = self.accumulator;
    relu(&mut out);
    out
  }
}

struct Layer<const INPUTS: usize, const OUTPUTS: usize> {
  w: [[f32; INPUTS]; OUTPUTS],
  b: [f32; OUTPUTS],
}

impl <const INPUTS: usize, const OUTPUTS: usize> Layer<INPUTS, OUTPUTS> {
  pub fn new(w: [[f32; INPUTS]; OUTPUTS], b: [f32; OUTPUTS]) -> Layer<INPUTS, OUTPUTS> {
    Layer { w, b }
  }

  pub fn forward(&mut self, inputs: [f32; INPUTS]) -> [f32; OUTPUTS] {
    let mut c = self.b;
    for w in 0..self.w.len() {
      c[w] += dot(&inputs, &self.w[w]);
    }
    c
  }
}
