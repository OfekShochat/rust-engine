extern crate chess;

pub mod psqt;
mod search;

fn main() {
  let s = search::Manager::new();
  s.iterative_deepening();
}
