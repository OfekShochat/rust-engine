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
  w1: [f32; 10],
}

impl Net {
  pub fn new(w1: [f32; 10]) -> Net {
    Net { w1 }
  }

  fn forward(&self, inputs: [f32; 10]) -> f32 {
    dot(&inputs, &self.w1)
  }
}