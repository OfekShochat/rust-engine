#![feature(array_zip)]
extern crate chess;
extern crate packed_simd;

pub mod psqt;
pub mod net;
pub mod nn;
mod movepick;
mod search;
mod uci;

fn main() {
  let mut ucih = uci::Uci::new();
  ucih.main();
}
