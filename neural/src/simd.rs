use std::{f64, mem};
#[cfg(target_arch = "x86_64")]
#[warn(non_snake_case)]
use std::arch::x86_64::*;

pub struct Layer {
  weights: Vec<Vec<__m256d>>,
  biases: Vec<Vec<__m256d>>
}

impl Layer {
  pub fn new() -> Self {
    return Self { weights: Vec::<Vec<__m256d>>::new(), biases: Vec::<Vec<__m256d>>::new() };
  }
  pub fn load_parameters(&mut self, w: Vec<Vec<f64>>, b: Vec<f64>) {
    let s = w.len();
    self.weights = self.split_more_and_set(w, 0.0);
    self.biases = self.split_more_and_set(Layer::reconstruct_biases(s, b), 0.0);
  }

  fn reconstruct_biases(lw: usize, b: Vec<f64>) -> Vec<Vec<f64>> {
    let mut new_biases: Vec<Vec<f64>> = [].to_vec();
    for _ in 0..b.len() {
      let mut temp: Vec<f64> = [].to_vec();
      for i in 0..lw {
        temp.push(b[i]);
      }
      new_biases.push(temp);
    }
    return new_biases;
  }

  fn reconstruct_inputs(lw: usize, mut x: Vec<f64>) -> Vec<Vec<f64>> {
    let mut r: Vec<Vec<f64>> = [].to_vec();
    for _ in 0..lw {
      let mx = &mut x;
      r.push(mx.to_vec());
    }
    return r;
  }

  #[allow(non_snake_case)]
  unsafe fn vector_term(
    xs: __m256d,
    ws: __m256d,
    bs: __m256d,
  ) -> (f64, f64, f64, f64) {
    let WxX = _mm256_mul_pd(xs, ws);
    let WxXpB = _mm256_add_pd(WxX, bs);
    let out_unpacked: (f64, f64, f64, f64) = mem::transmute(WxXpB);
    return out_unpacked;
  }

  pub fn forward(&mut self, x: Vec<f64>) -> Vec<f64> {
    let inputs = self.split_more_and_set(Layer::reconstruct_inputs(self.weights.len(), x), 0.0);
    let mut out: Vec<f64> = [].to_vec();
    //assert!(x.len() == self.biases.len());
    for i in 0..self.weights.len() {
      for j in 0..self.weights[i].len() {
        unsafe {
          let r = Layer::vector_term(inputs[i][j], self.weights[i][j], self.biases[i][j]);
          out.push(r.0 + r.1 + r.2 + r.3);
        }
      }
    }
    return out;
  }

  fn split_more_and_set(&mut self, a: Vec<Vec<f64>>, left_over: f64) -> Vec<Vec<__m256d>> {
    // splitting weights to groups of 4
    let mut splitted = Vec::new();
    
    for i in 0..a.len() {
      let mut curr_splitted = Vec::new();
      let mut temp = Vec::new();
      for j in 0..a[i].len() {
        temp.push(a[i][j]);
        if j % 4 == 3 {
          unsafe {
            curr_splitted.push(_mm256_set_pd(temp[0], temp[1], temp[2], temp[3]));
          }
          temp.clear()
        }
      }
      if temp.len() > 0 {
        for _ in 0..4-temp.len() {
          temp.push(left_over)
        }
        unsafe {
          curr_splitted.push(_mm256_set_pd(temp[0], temp[1], temp[2], temp[3]));
        }
      }
      splitted.push(curr_splitted);
    }
    return splitted;
  }

  /*fn split_and_set(&mut self, a: Vec<f64>, left_over: f64) -> Vec<__m256d> {
    // splitting weights to groups of 4
    let mut splitted = Vec::new();
    let mut temp = Vec::new();
    for i in 0..a.len() {
      temp.push(a[i]);
      if i % 4 == 3 {
        unsafe {
          splitted.push(_mm256_set_pd(temp[0], temp[1], temp[2], temp[3]));
        }
        temp.clear()
      }
    }
    if temp.len() > 0 {
      for _ in 0..4-temp.len() {
        temp.push(left_over)
      }
      unsafe {
        splitted.push(_mm256_set_pd(temp[0], temp[1], temp[2], temp[3]));
      }
    }
    return splitted;
  }
  */
}