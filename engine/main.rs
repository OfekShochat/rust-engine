#![feature(array_zip)]
extern crate chess;
extern crate packed_simd;

mod movepick;
pub mod net;
pub mod nn;
pub mod psqt;
mod search;
mod uci;

fn main() {
  let mut ucih = uci::Uci::new();
  ucih.main();
}
