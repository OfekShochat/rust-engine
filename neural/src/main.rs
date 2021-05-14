mod simd;

fn main() {
  let mut l = simd::Layer::new();
  let mut w = Vec::<Vec<f64>>::new();
  w.push([0.1, 0.2, 0.3].to_vec());
  w.push([0.14325, 0.1314565, 0.123214].to_vec());
  let mut b: Vec<f64> = [].to_vec();
  b.push(0.4);
  b.push(0.2);
  b.push(0.5);
  l.load_parameters(w, b);
  let test: Vec<f64> = [0.1, 0.4, 0.4].to_vec();
  println!("{}", l.forward(test)[0]);
}