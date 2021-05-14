extern crate npy;

use std::io::Read;
use npy::NpyData;

const fn load(const path: str) {
  let mut buf = vec![];
  unsafe {
    std::fs::File::open(path).unwrap()
      .read_to_end(&mut buf).unwrap();
  }
  let data: NpyData<f64> = NpyData::from_bytes(&buf).unwrap();
  return data;
}