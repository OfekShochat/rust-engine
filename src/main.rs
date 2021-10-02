extern crate chess;

pub mod psqt;
mod search;
mod movepick;

fn main() {
  let s = search::Manager::new();
  s.start();
}
