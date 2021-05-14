mod simd;
use std::time::SystemTime;

fn main() {
  let mut l = simd::Layer::new();
  let mut w = Vec::<Vec<f64>>::new();
  w.push([0.141325, 0.043141, 0.14341, 0.141325, 0.043141, 0.043141].to_vec());
  w.push([0.14325, 0.1314565, 0.123214, 0.141325, 0.043141, 0.043141].to_vec());
  w.push([0.14325, 0.1314565, 0.123214, 0.141325, 0.043141, 0.043141].to_vec());
  w.push([0.14325, 0.1314565, 0.123214, 0.141325, 0.043141, 0.043141].to_vec());
  w.push([0.14325, 0.1314565, 0.123214, 0.141325, 0.043141, 0.043141].to_vec());
  let mut b: Vec<f64> = [].to_vec();
  b.push(0.1431513);
  b.push(0.156635);
  b.push(0.163564);
  b.push(0.163564);
  b.push(0.163564);
  l.load_parameters(w, b);
  let test: Vec<f64> = [0.154151, 0.1431413, 0.22141231, 0.154151, 0.1431413, 0.22141231].to_vec();
  let start = SystemTime::now();
  let out = l.forward(test);
  let duration: u128 = start.elapsed().unwrap().as_nanos();
  for i in 0..out.len() {
    println!("{}", out[i]);
  }
  println!("{}", duration)
}