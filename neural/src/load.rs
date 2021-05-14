extern crate npy;
use std::io::Read;

pub fn load() -> (Vec<Vec<f64>>, Vec<f64>) {
  let mut buf = vec![];
  let mut biases = vec![];
  let mut weights = vec![];
  std::fs::File::open(env!("nnPath")).unwrap()
    .read_to_end(&mut buf).unwrap();
  let data: npy::NpyData<f64> = npy::NpyData::from_bytes(&buf).unwrap();
  let mut a: i8 = 0;
  let mut weights_per_neuron = 0.0;
  let mut temp = vec![];
  let mut is_biases: bool = false;
  for i in data {
    if a < 1 {
      weights_per_neuron = i;
      a += 1;
      continue;
    }
    if i == 8.36 {
      is_biases = true;
      a += 1;
      continue;
    }
    if !is_biases {
      if !((a-1) % (weights_per_neuron as i8) == (weights_per_neuron as i8 - 1)) {
        temp.push(i);
      } else {
        temp.push(i);
        weights.push(temp.clone());
        temp.clear();
      }
    } else {
      if i == 8.366 {
        break;
      }
      biases.push(i);
    }
    a+=1;
  }
  return (weights, biases);
}