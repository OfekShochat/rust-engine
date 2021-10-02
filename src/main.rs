extern crate chess;

pub mod psqt;
mod search;
mod movepick;
mod uci;

fn main() {
  let mut ucih = uci::Uci::new();
  ucih.main();
}
