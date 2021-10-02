use std::{io::stdin, str::SplitAsciiWhitespace};

use crate::search::{Limit, Manager};

pub struct Uci {
  searcher: Manager,
  position_fen: String,
}

impl Uci {
  pub fn new() -> Uci {
    Uci {
      searcher: Manager::new(),
      position_fen: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string(),
    }
  }

  pub fn main(&mut self) {
    loop {
      let mut buf = String::new();
      stdin().read_line(&mut buf).unwrap();
      let mut tokens = buf.split_ascii_whitespace();
      match tokens.next().unwrap() {
        "go" => self.searcher.start(self.position_fen.clone(), Limit::timed(10000)),
        "position" => self.position(&mut tokens),
        _ => eprintln!("invalid command"),
      }
    }
  }

  fn position(&mut self, buf: &mut SplitAsciiWhitespace<'_>) {
    match buf.next().unwrap() {
      "fen" => {
        self.position_fen = String::new();
        while let Some(a) = buf.next() {
          self.position_fen += &(a.to_owned() + " ");
        }
      }
      _ => {}
    }
  }
}
