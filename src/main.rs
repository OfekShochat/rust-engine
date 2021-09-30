use std::time::Instant;

extern crate chess;

mod search;

fn main() {
  let mut s = search::SearchWorker::new();
  let start = Instant::now();
  let r = s.search(chess::Board::default(), -10000, 10000, 10);
  println!(
    "cp {} nps {} time {}",
    r,
    s.nodes as f32 / start.elapsed().as_secs_f32(),
    start.elapsed().as_secs_f32()
  );
}
