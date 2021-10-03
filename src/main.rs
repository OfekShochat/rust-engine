extern crate chess;
extern crate packed_simd;

pub mod psqt;
mod movepick;
mod search;
mod uci;
mod simd;
mod nn;

fn main() {
  let mut ucih = uci::Uci::new();
  ucih.main();
}
