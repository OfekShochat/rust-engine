extern crate chess;

mod search;

pub const INF: i32 = 10000;

fn main() {
  let mut s = search::SearchWorker::new();
  s.iterative_deepening(chess::Board::default(), -INF, INF, 100);
}
