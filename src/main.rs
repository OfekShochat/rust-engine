extern crate chess;

mod search;

fn main() {
  let s = search::Manager::new();
  s.iterative_deepening();
}
