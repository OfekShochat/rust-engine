extern crate chess;

mod movepick;
pub mod psqt;
mod search;
mod uci;

fn main() {
  let mut ucih = uci::Uci::new();
  ucih.main();
}
