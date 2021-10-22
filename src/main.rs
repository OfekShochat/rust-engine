#![feature(array_zip)]
extern crate chess;
extern crate packed_simd;

mod movepick;
pub mod net;
pub mod nn;
mod search;
pub mod search_consts;
pub mod position;
mod uci;
mod transpositions;

fn main() {
  let mut ucih = uci::Uci::new(4);
  ucih.main();
}
