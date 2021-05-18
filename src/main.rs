mod uci;
pub mod search;
pub mod thread;
extern crate chess;

#[allow(unused_must_use)]
fn main() {
  let mut uci_handle = uci::UciFunctions::new();
  uci_handle.go(100, -1, -1, false);
}