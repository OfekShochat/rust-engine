extern crate num_cpus;
extern crate chess;
use sthread;
use std::{time::SystemTime, vec};
use std::collections::HashMap;
use std::sync::{mpsc, Arc};
use std::sync::atomic::{AtomicBool, AtomicI32};
use std::thread;

pub struct Search {
  nodes: i32,
  tt   : HashMap<i32, sthread::TtEntry>,
  history
       : sthread::HistoryHeuristics,
  //thread_pool: rayon::ThreadPoolBuilder
  pruned: i32,
  receivers: Vec<mpsc::Receiver<sthread::ThreadMessage>>,
  senders: Vec<mpsc::Sender<sthread::ThreadMessage>>,
  threads: Vec<sthread::ThreadManager>
}

impl Search {
  pub fn new() -> Self {
    // setup counter_moves heuristic table
    let mut cm: Vec<Vec<i32>> = vec![];
    let mut ks: Vec<Vec<chess::ChessMove>> = vec![];
    for _ in 0..64 {
      cm.push([1; 64].to_vec());
      ks.push([chess::ChessMove::new(chess::Square::A1, chess::Square::A1, None); 64].to_vec());
    }

    let mut recvs: Vec<mpsc::Receiver<sthread::ThreadMessage>> = Vec::<mpsc::Receiver::<sthread::ThreadMessage>>::new();
    let mut sends: Vec<mpsc::Sender<sthread::ThreadMessage>> = Vec::<mpsc::Sender::<sthread::ThreadMessage>>::new();
    let mut threads: Vec<sthread::ThreadManager> = vec![];

    let cpus = num_cpus::get_physical();
    println!("Detected {} cores", cpus);

    let should_stop = Arc::new(AtomicBool::new(false));

    for id in 0..cpus {
      let (send, recv) = mpsc::channel();
      recvs.push(recv);
      sends.push(send);
      threads.push(sthread::ThreadManager { id: id, nodes: Arc::new(AtomicI32::new(0)), should_stop: should_stop.clone(), sender: send, score: 0, history: sthread::HistoryHeuristics { counter_moves: cm, killers: ks } });
    }

    return Self { nodes: 0, tt: HashMap::new(), history: sthread::HistoryHeuristics { counter_moves: cm, killers: ks }, pruned: 0, receivers: recvs, senders: sends, threads: threads };
  }

pub fn search_pos(&mut self, board: chess::Board, alpha: i32, beta: i32) {
  let mut iterable = chess::MoveGen::new_legal(&board);
  for t in 0..self.receivers.len() {
    let mut result: chess::Board = board.clone();
    board.make_move(m, &mut result);
    thread::spawn(move || {
      sthread::iterative_deepening(*mut self.threads[t], result, alpha, beta);
    });
  }
}

/*
pub fn iterative_deepening(&mut self, board: chess::Board, alpha: i32, beta: i32, stopper: Stopper) -> (i32, chess::ChessMove) {
  let mut result: (i32, chess::ChessMove) = (0, chess::ChessMove::new(chess::Square::A1, chess::Square::A1, None));
  let start = SystemTime::now();
  for d in 2..stopper.depth+1 {
    result = self.search(board, d as i32, alpha, beta, stopper);
    let duration: u128 = start.elapsed().unwrap().as_millis();
    println!("info depth {} score cp {} nodes {} pruned {} nps {} time {} pv {}", d, result.0, self.nodes, self.pruned, (self.nodes as f32 /((duration as f32 /1000 as f32)) as f32) as i32, duration, result.1);
  }
  self.nodes = 0;

  return result;
}

fn search(&mut self, board: chess::Board, depth: i32, mut alpha: i32, beta: i32, stopper: Stopper) -> (i32, chess::ChessMove) {
  let mut iterable = self.order(board);
  let color = if board.side_to_move() == chess::Color::Black {-1} else {1};  
  let mut best: chess::ChessMove = chess::ChessMove::default();
  for m in &mut iterable {
    if stopper.should_stop {
      break;
    }
    let mut result: chess::Board = board.clone();
    board.make_move(m, &mut result);
    let r: i32 = -self.alpha_beta(result, 0, depth, -beta, -alpha, -color, stopper, 0);
    //println!("{} {}", m, r);
    if r >= beta {
      return (beta, m);
    }
    if r > alpha {
      alpha = r;
      best = m;
    }
  }
  return (alpha, best);
}

fn add_to_tt(&mut self, pos: chess::Board, entry: TtEntry) {
  let h: i32 = pos.get_hash() as i32;
  self.tt.insert(h, entry);
}*/
}