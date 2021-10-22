extern crate attohttpc;

use attohttpc::get;
use std::{fs::File, io::Write};

const NET_URI: &str = "https://raw.githubusercontent.com/ofekshochat/sunset-networks/main/net.rs";

fn main() {
  let resp = get(NET_URI).send();
  match resp {
    Ok(resp) => {
      let mut file = File::create("src/net.rs").unwrap();
      file.write_all(&resp.bytes().unwrap()).unwrap();
    }
    _ => panic!("Failed to retrieve network.")
  }
}