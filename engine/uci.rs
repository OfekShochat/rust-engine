use std::{
  io::stdin,
  str::{FromStr, SplitAsciiWhitespace},
};

use chess::{Board, ChessMove};

use crate::search::{Limit, Manager};

const STARTPOS: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

pub struct Uci {
  searcher: Manager,
  position_fen: String,
  threads: usize,
}

impl Uci {
  pub fn new(threads: usize) -> Uci {
    Uci {
      searcher: Manager::new(),
      position_fen: STARTPOS.to_string(),
      threads,
    }
  }

  pub fn main(&mut self) {
    loop {
      let mut buf = String::new();
      stdin().read_line(&mut buf).unwrap();
      let mut tokens = buf.split_ascii_whitespace();
      match tokens.next() {
        Some("go") => self.go(&mut tokens),
        Some("position") => self.position(&mut tokens),
        Some("isready") => println!("readyok"),
        Some("uci") => self.uci(),
        Some("quit") => break,
        _ => eprintln!("invalid command"),
      }
    }
  }

  fn go(&mut self, tokens: &mut SplitAsciiWhitespace) {
    match tokens.next() {
      Some("time") => self.searcher.start(
        self.position_fen.clone(),
        Limit::timed(tokens.next().unwrap().parse().unwrap()),
        self.threads,
      ),
      Some("depth") => self.searcher.start(
        self.position_fen.clone(),
        Limit::depthed(tokens.next().unwrap().parse().unwrap()),
        self.threads,
      ),
      _ => self
        .searcher
        .start(self.position_fen.clone(), Limit::timed(360000), self.threads),
    }
  }

  fn position(&mut self, tokens: &mut SplitAsciiWhitespace) {
    match tokens.next().unwrap() {
      "fen" => {
        self.position_fen = String::new();
        self.parse_fen(tokens);
      }
      "startpos" => {
        self.position_fen = STARTPOS.to_string();
        self.parse_fen(tokens);
      }
      _ => {}
    }
  }

  fn parse_fen(&mut self, tokens: &mut SplitAsciiWhitespace) {
    while let Some(a) = tokens.next() {
      if a == "moves" {
        break;
      }
      self.position_fen += &(a.to_owned() + " ");
    }
    let mut b = Board::from_str(&self.position_fen).unwrap();
    while let Some(m) = tokens.next() {
      b = b.make_move_new(ChessMove::from_str(m).expect("illegal move"));
    }
    self.position_fen = b.to_string();
  }

  fn uci(&mut self) {
    println!("id name ");
    println!("id author Ofek Shochat");
    println!("uciok");
  }
}
