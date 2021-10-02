use std::{io::stdin, str::SplitAsciiWhitespace};

use crate::search::Manager;

pub struct Uci {
  searcher: Manager,
  position_fen: String,
}

impl Uci {
  pub fn new() -> Uci {
    Uci {
      searcher: Manager::new(),
      position_fen: String::new(),
    }
  }

  pub fn main(&mut self) {
    loop {
      let mut buf = String::new();
      stdin().read_line(&mut buf).unwrap();
      let mut tokens = buf.split_ascii_whitespace();
      match tokens.next().unwrap() {
        "go" => self.searcher.start(self.position_fen.clone()),
        "position" => self.position(&mut tokens),
        _ => eprintln!("invalid command"),
      }
    }
  }

  fn position(&mut self, buf: &mut SplitAsciiWhitespace<'_>) {
    match buf.next().unwrap() {
      "fen" => {
        while let Some(a) = buf.next() {
          self.position_fen += &(a.to_owned() + " ");
        }
      }
      _ => {}
    }
  }
}
