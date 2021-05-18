mod simd;
mod load;
use std::time::SystemTime;
extern crate rand;
use rand::Rng;

fn main() {
  let d = load::load();
  /*for i in 0..d.0.len() {
    for j in 0..d.0.len() {
      println!("{}", d.0[i][j])
    }
  }*/
  //println!("");
  let mut l = simd::Layer::new();
  let /*mut*/ w = d.0; // Vec::<Vec<f32>>::new();
  let /*mut*/ b: Vec<f32> = d.1; //[].to_vec();
  //for i in b.clone() {
  //  println!("{}", i)
  //}
  //println!("");
  let mut rng = rand::thread_rng();

  l.load_parameters(w, b);
  let test: Vec<f32> = [rng.gen::<f32>()*2.0-1.0, rng.gen::<f32>()*2.0-1.0, rng.gen::<f32>()*2.0-1.0, rng.gen::<f32>()*2.0-1.0, rng.gen::<f32>()*2.0-1.0].to_vec();
  let start = SystemTime::now();
  let out = l.forward(test);
  let o = simd::Layer::clamp(out, -1.0, 1.0);
  let duration: u128 = start.elapsed().unwrap().as_nanos();
  for i in 0..o.len() {
    println!("{}", o[i]);
  }
  println!("{}", duration)
}