mod simd;
mod load;
use std::time::SystemTime;

fn main() {
  let d = load::load();
  for i in 0..d.1.len() {
    println!("{}", d.1[i])
  }
  let mut l = simd::Layer::new();
  let mut w = Vec::<Vec<f64>>::new();
  w.push([0.141325, 0.043141, -0.14341, 0.141325, 0.043141, 0.043141].to_vec());
  w.push([0.14325, 0.1314565, 0.123214, 0.141325, 0.043141, 0.043141].to_vec());
  w.push([0.14325, 0.1314565, 0.123214, 0.141325, 0.043141, 0.043141].to_vec());
  w.push([0.14325, 0.1314565, 0.123214, 0.141325, 0.043141, 0.043141].to_vec());
  w.push([0.256652, 0.1314565, 0.123214, 0.141325, 0.043141, 0.043141].to_vec());
  let mut b: Vec<f64> = [].to_vec();
  b.push(0.26424);
  b.push(0.89878927);
  b.push(0.163564);
  b.push(0.163564);
  b.push(0.163564);
  l.load_parameters(w, b);
  let test: Vec<f64> = [0.154151, 0.1431413, 0.22141231, 0.154151, 0.1431413, 0.22141231].to_vec();
  let start = SystemTime::now();
  let out = l.forward(test);
  let o = simd::Layer::clamp(out, -1.0, 1.0);
  let duration: u128 = start.elapsed().unwrap().as_nanos();
  //for i in 0..o.len() {
  //  println!("{}", o[i]);
  //}
  println!("{}", duration)
}