use std::{f32, mem};
#[cfg(target_arch = "x86_64")]
#[warn(non_snake_case)]
use std::arch::x86_64::*;
use std::time::SystemTime;

pub struct Network {
  hidden1: Layer,
  hidden2: Layer,
  output:  Layer
}

impl Network {
  pub fn new() -> Self {
    return Self { hidden1: Layer::new(), hidden2: Layer::new(), output: Layer::new() }
  }

  pub fn forward() {
    
  }
}

pub struct Layer {
  weights: Vec<Vec<__m256>>,
  biases: Vec<Vec<__m256>>
}

impl Layer {
  pub fn new() -> Self {
    return Self { weights: Vec::<Vec<__m256>>::new(), biases: Vec::<Vec<__m256>>::new() };
  }
  pub fn load_parameters(&mut self, w: Vec<Vec<f32>>, b: Vec<f32>) {
    let s = w.len();
    self.weights = self.split_more_and_set(w, 0.0);
    self.biases = self.split_more_and_set(Layer::reconstruct_biases(s, b), 0.0);
  }

  pub fn clamp(out: Vec<f32>, min: f32, max: f32) -> Vec<f32> {
    let mut o: Vec<f32> = [].to_vec();
    for i in 0..out.len() {
      if out[i] > max {
        o.push(max);
      } else if out[i] < min {
        o.push(min);
      } else {
        o.push(out[i]);
      }
    }
    return o;
  }

  fn reconstruct_biases(lw: usize, b: Vec<f32>) -> Vec<Vec<f32>> {
    let mut new_biases: Vec<Vec<f32>> = [].to_vec();
    for i in 0..b.len() {
      let mut temp: Vec<f32> = [].to_vec();
      for j in 0..lw {
        if j == 0 {
          temp.push(b[i]);
        } else {
          temp.push(0.0);
        }
      }
      new_biases.push(temp);
    }
    return new_biases;
  }

  fn reconstruct_inputs(lw: usize, mut x: Vec<f32>) -> Vec<Vec<f32>> {
    let mut r: Vec<Vec<f32>> = [].to_vec();
    for _ in 0..lw {
      let mx = &mut x;
      r.push(mx.to_vec());
    }
    return r;
  }

  #[allow(non_snake_case)]
  unsafe fn vector_term(
    xs: __m256,
    ws: __m256,
    bs: __m256,
  ) -> (f32, f32, f32, f32, f32, f32, f32, f32) {
    let WxX = _mm256_mul_ps(xs, ws);
    let WxXpB = _mm256_add_ps(WxX, bs);
    let out_unpacked: (f32, f32, f32, f32, f32, f32, f32, f32) = mem::transmute(WxXpB);
    return out_unpacked;
  }

  pub fn forward(&mut self, x: Vec<f32>) -> Vec<f32> {
    //let start = SystemTime::now();
    let inputs = self.split_more_and_set(Layer::reconstruct_inputs(self.weights.len(), x), 0.0);
    //let duration: u128 = start.elapsed().unwrap().as_nanos();
    //println!("{}", duration);
    let mut out: Vec<f32> = [].to_vec();
    assert!(self.weights.len() == self.biases.len());
    for i in 0..self.weights.len() {
      let mut neuron_out: f32 = 0.0;
      for j in 0..self.weights[i].len() {
        unsafe {
          let r = Layer::vector_term(inputs[i][j], self.weights[i][j], self.biases[i][j]);
          neuron_out += r.0 + r.1 + r.2 + r.3 + r.4 + r.5 + r.6 + r.7;
        }
      }
      out.push(neuron_out);
    }
    return out;
  }

  fn split_more_and_set(&mut self, a: Vec<Vec<f32>>, left_over: f32) -> Vec<Vec<__m256>> {
    // splitting weights to groups of 8 for simd calculation
    let mut splitted = Vec::new();
    
    for i in 0..a.len() {
      let mut curr_splitted = Vec::new();
      let mut iter = a[i].chunks_exact(8);
      while let Some(chunk) = iter.next() {
        unsafe {
          curr_splitted.push(_mm256_set_ps(chunk[0], chunk[1], chunk[2], chunk[3], chunk[4], chunk[5], chunk[6], chunk[7]));
        }
      }
      let rest = iter.remainder();
      let rl = rest.len();
      if rl == 1 {
        unsafe {
          curr_splitted.push(_mm256_set_ps(rest[0], left_over, left_over, left_over, left_over, left_over, left_over, left_over));
        }
      } else if rl == 2 {
        unsafe {
          curr_splitted.push(_mm256_set_ps(rest[0], rest[1], left_over, left_over, left_over, left_over, left_over, left_over));
        }
      } else if rl == 3 {
        unsafe {
          curr_splitted.push(_mm256_set_ps(rest[0], rest[1], rest[2], left_over, left_over, left_over, left_over, left_over));
        }
      } else if rl == 4 {
        unsafe {
          curr_splitted.push(_mm256_set_ps(rest[0], rest[1], rest[2], rest[3], left_over, left_over, left_over, left_over));
        }
      } else if rl == 5 {
        unsafe {
          curr_splitted.push(_mm256_set_ps(rest[0], rest[1], rest[2], rest[3], rest[4], left_over, left_over, left_over));
        }
      } else if rl == 6 {
        unsafe {
          curr_splitted.push(_mm256_set_ps(rest[0], rest[1], rest[2], rest[3], rest[4], rest[5], left_over, left_over));
        }
      } else if rl == 7 {
        unsafe {
          curr_splitted.push(_mm256_set_ps(rest[0], rest[1], rest[2], rest[3], rest[4], rest[5], rest[6], left_over));
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