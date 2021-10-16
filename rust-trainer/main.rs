extern crate chess;
extern crate easy_reader;
extern crate fastapprox;
extern crate serde_derive;
extern crate tch;
extern crate toml;
extern crate ctrlc;

use chess::{Board, Color};
use easy_reader::EasyReader;
use fastapprox::fast::sigmoid;
use serde_derive::Deserialize;
use std::io::Read;
use std::collections::HashMap;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::time::Duration;
use std::{fs::File, str::FromStr};
use tch::{
  nn::{self, Module, OptimizerConfig},
  Device, Tensor,
};
use ctrlc::set_handler;
use toml::from_str;
use std::fs;
use std::process::exit;

const DEADLINE: u64 = 100;
const SCALE: f32 = 1024.0;

fn create_net(vs: &nn::Path) -> impl Module {
  nn::seq()
    .add(nn::linear(vs / "fc0", 768, 128, Default::default()))
    .add_fn(|xs| xs.relu())
    .add(nn::linear(vs / "fc1", 128, 1, Default::default()))
    .add_fn(|xs| xs.relu())
}

fn main() {
  let mut cfg_str = "".to_string();
  File::open("config.toml")
    .unwrap()
    .read_to_string(&mut cfg_str)
    .unwrap();
  let config: Config = from_str(&cfg_str).unwrap();

  let vs = nn::VarStore::new(Device::cuda_if_available());
  let net = create_net(&vs.root());

  let mut opt = nn::AdamW::default().build(&vs, config.training.lr).unwrap();
  let mut data = Data::new(config.training.batch_size);
  data.start(config.workers);
  let mut running_loss = Tensor::of_slice(&[0.0])
    .to_device(Device::cuda_if_available())
    .detach();

  let output_path = config.output_path.clone();
  set_handler(move || {
    save(&output_path, vs.variables());
    exit(0);
  }).expect("Could not setup ctrl-c handler.");
  for (step, (x, y)) in (&mut data).enumerate() {
    let loss = net.forward(&x).mse_loss(&y, tch::Reduction::Mean);
    opt.backward_step(&loss);
    running_loss += &loss.detach();

    if step % config.report_freq == config.report_freq - 1 {
      println!(
        "step {} loss {:?}",
        step + 1,
        (&running_loss /
          Tensor::of_slice(&[config.report_freq as f32])
            .to_device(Device::cuda_if_available())
            .detach())
      );
      running_loss = Tensor::of_slice(&[0.0])
        .to_device(Device::cuda_if_available())
        .detach();
    }
    drop(x);
    drop(y);
  }
}

fn save(output_path: &str, variables: HashMap<String, Tensor>) {
  let mut out = "".to_string();
  for (name, param) in variables {
    let input_size = match name.as_str() {
      "fc0.weight" => 768,
      "fc1.weight" => 128,
      _ => 0,
    };
    let layer_size = match name.as_str() {
      "fc0.weight" => 128,
      "fc0.bias" => 128,
      "fc1.weight" => 1,
      "fc1.bias" => 1,
      _ => unreachable!(),
    };
    let name = name.replace(".", "_").to_uppercase();

    if input_size != 0 {
      out += &format!("pub const {}: [[f32; {}]; {}] = {:?};\n", name, input_size, layer_size, Vec::<Vec<f32>>::from(param.detach().contiguous().view([-1, input_size])))
    } else {
      out += &format!("pub const {}: [f32; {}] = {:?};\n", name, layer_size, Vec::<Vec<f32>>::from(param.detach().contiguous()).into_iter().flatten().collect::<Vec<f32>>())
    }
  }
  fs::write(output_path, out).expect("Couldn't open file when saving.");
  println!("Saved weights to {}.", output_path);
}

#[derive(Deserialize)]
struct Training {
  batch_size: usize,
  lr: f64,
}

#[derive(Deserialize)]
struct Config {
  training: Training,
  workers: usize,
  output_path: String,
  report_freq: usize,
}

struct Datapoint {
  board: [f32; 768],
  eval: f32,
}

impl Datapoint {
  pub fn from_string(line: String) -> Option<Datapoint> {
    let parts: Vec<&str> = line.split("|").collect();
    let board = Board::from_str(&parts[0]);
    if board.is_err() {
      return None;
    }
    let board = board.unwrap();

    let mut inputs = [0.0; 768];
    match board.side_to_move() {
      Color::White => {
        for s in chess::ALL_SQUARES {
          let color = board.color_on(s);
          let piece = board.piece_on(s);

          match color {
            Some(chess::Color::White) => inputs[piece.unwrap().to_index()] = 1.0,
            Some(chess::Color::Black) => inputs[(piece.unwrap().to_index() + 6) % 12] = 1.0,
            None => continue,
          }
        }
      }
      Color::Black => {
        for s in chess::ALL_SQUARES {
          let color = board.color_on(s);
          let piece = board.piece_on(s);

          match color {
            Some(chess::Color::White) => inputs[(piece.unwrap().to_index() + 6) % 12] = 1.0,
            Some(chess::Color::Black) => inputs[piece.unwrap().to_index()] = 1.0,
            None => continue,
          }
        }
      }
    }
    let e: Result<f32, _> = parts[1].parse();
    if e.is_ok() {
      let e = e.unwrap() / SCALE;
      Some(Datapoint {
        board: inputs,
        eval: e,
      })
    } else {
      None
    }
  }
}

fn data_worker(sender: Sender<Datapoint>) {
  let file = File::open("data.txt").unwrap();
  let mut reader = EasyReader::new(file).unwrap();
  loop {
    let l = reader.random_line();
    if l.is_err() {
      continue;
    }
    let dp = Datapoint::from_string(l.unwrap().unwrap());
    if dp.is_some() {
      sender.send(dp.unwrap()).unwrap();
    }
  }
}

struct Data {
  send: Sender<Datapoint>,
  recv: Receiver<Datapoint>,
  batch_size: usize,
}

impl Data {
  pub fn new(batch_size: usize) -> Data {
    let (send, recv) = channel();
    Data {
      send,
      recv,
      batch_size,
    }
  }

  pub fn start(&mut self, workers: usize) {
    for _ in 0..workers {
      let sender_cp = self.send.clone();
      thread::spawn(move || {
        data_worker(sender_cp);
      });
    }
  }
}

impl Iterator for Data {
  type Item = (tch::Tensor, tch::Tensor);
  fn next(&mut self) -> Option<(tch::Tensor, tch::Tensor)> {
    let mut batch = vec![];
    let mut targets = Vec::with_capacity(self.batch_size);
    for _ in 0..self.batch_size {
      let s = self.recv.recv_timeout(Duration::from_millis(DEADLINE));
      if s.is_ok() {
        let s = s.unwrap();
        batch.extend(s.board.iter().cloned());
        targets.push(sigmoid(s.eval));
      } else {
        return None;
      }
    }
    Some((
      Tensor::of_slice(&batch)
        .view_(&[-1, 768])
        .to_device(Device::cuda_if_available())
        .detach(),
      Tensor::of_slice(&targets)
        .view_(&[-1, 1])
        .to_device(Device::cuda_if_available())
        .detach(),
    ))
  }
}
