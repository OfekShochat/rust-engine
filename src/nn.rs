use packed_simd::f32x4;

fn dot(a: &[f32], b: &[f32]) -> f32 {
  assert_eq!(a.len(), b.len());
  assert!(a.len() % 4 == 0);

  a.chunks_exact(4)
    .map(f32x4::from_slice_unaligned)
    .zip(b.chunks_exact(4).map(f32x4::from_slice_unaligned))
    .map(|(a, b)| a * b)
    .sum::<f32x4>()
    .sum()
}

struct Net {
  w1: [[f32; 718]; 256],
  w2: [[f32; 256]; 128],
  w3: [[f32; 128]; 32],
  w4: [[f32; 32]; 1],
  b1: [f32; 256],
  b2: [f32; 128],
  b3: [f32; 32],
  b4: [f32; 1],
}

impl Net {
  pub fn new(w1: [[f32; 718]; 256],
             w2: [[f32; 256]; 128],
             w3: [[f32; 128]; 32],
             w4: [[f32; 32]; 1],
             b1: [f32; 256],
             b2: [f32; 128],
             b3: [f32; 32],
             b4: [f32; 1],
            ) -> Net {
    Net { w1, w2, w3, w4, b1, b2, b3, b4 }
  }

  pub fn forward(&self, inputs: [f32; 718]) -> f32 {
    let mut b = self.b1.clone();
    for w in 0..self.w1.len() {
      b[w] += dot(&inputs, &self.w1[w]);
    }
    let mut b = self.b2.clone();
    for w in 0..self.w2.len() {
      b[w] += dot(&inputs, &self.w2[w]);
    }
    let mut b = self.b3.clone();
    for w in 0..self.w3.len() {
      b[w] += dot(&inputs, &self.w3[w]);
    }
    let mut b = self.b4.clone();
    for w in 0..self.w4.len() {
      b[w] += dot(&inputs, &self.w4[w]);
    }
    unsafe { *b.get_unchecked(0) }
  }
}