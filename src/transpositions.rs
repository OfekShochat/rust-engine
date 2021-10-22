use chess::ChessMove;
use std::sync::atomic::{AtomicPtr, Ordering::Acquire};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
pub struct TTEntry {
  pub mov: ChessMove,
  depth: u8,
  score: i32,
}

pub struct TranspositionTable {
  tt: AtomicPtr<HashMap<u64, TTEntry>>,
}

impl TranspositionTable {
  pub fn new() -> TranspositionTable {
    TranspositionTable { tt: AtomicPtr::new(&mut HashMap::new()) }
  }

  pub fn insert(&mut self, hash: u64, depth: u8, mov: ChessMove, score: i32) {
    unsafe {
      self.tt.load(Acquire)
        .as_mut()
        .unwrap()
        .insert(
          hash,
          TTEntry { mov, depth, score }
      );
    }
  }
}