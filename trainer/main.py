import torch
from torch.functional import Tensor
import torch.nn as nn
from torch.utils.data import DataLoader, IterableDataset
import torch.nn.functional as F
from chess import BLACK, BaseBoard, SQUARES
from numpy import array
from random import choice, shuffle
from sys import exit as sys_exit, argv
from os import path
import wandb
wandb.init("rust-engine")

def cross_entropy_loss(x, p):
  return torch.mean(-(p*F.logsigmoid(x) + (1-p)*F.logsigmoid(-x)))

class Set(IterableDataset):
  def __init__(self) -> None:
    super(Set, self).__init__()

  def sample_iter(self) -> None:
    from glob import glob
    files = glob("./data/run3/*.txt")
    f = open(choice(files), "r").readlines()
    for i in shuffle(f):
      try:
        a = i.split("|")
        yield self.build(a[0][:a[0].find(" ")], "w" in a[0]), array((int(a[1]) + 0.5 * int(a[2])) / 1024)
      except:
        pass


  def build(self, s, flip) -> list[list[int]]:
    a = [0]*64*12
    b = BaseBoard(s)
    if flip:
      b = b.mirror()
    for i in SQUARES:
      p = b.piece_at(i)
      if p:
        if p.color == BLACK:
          a[(p.piece_type + 5) * i] = 1.0
        else:
          a[p.piece_type * i] = 1.0
    return array([a])

  def __iter__(self):
    for sample in self.sample_iter():
      yield sample

class NN(nn.Module):
  def __init__(self) -> None:
    super(NN, self).__init__()
    self.fc0 = nn.Linear(768, 256)
    self.fc1 = nn.Linear(256, 128)
    self.fc2 = nn.Linear(128, 32)
    self.fc3 = nn.Linear(32, 1)

    from adabelief_pytorch import AdaBelief
    self.optimizer = AdaBelief(self.parameters(), lr=1e-3, eps=1e-12, rectify=False, print_change_log=False)

  def forward(self, x) -> Tensor:
    x = F.relu(self.fc0(x))
    x = F.relu(self.fc1(x))
    x = F.relu(self.fc2(x))
    x = self.fc3(x)
    return x

  def train_step(self, inputs, y) -> float:
    self.optimizer.zero_grad()

    out = self(inputs.float())
    loss = cross_entropy_loss(out, y.float())
    loss.backward()

    self.optimizer.step()

    return loss

  def save(self, path) -> None:
    open(path, "w+").write("")
    f = open(path, "a+")
    f.write("#[rustfmt::skip]\n")
    for (name, param) in self.named_parameters():
      name = name.replace(".", "_")
      try:
        f.write(f"pub const {name.upper()}: [[f32; {len(param[0])}]; {len(param)}] = {param.tolist()};\n")
      except:
        f.write(f"pub const {name.upper()}: [f32; {len(param)}] = {param.tolist()};\n")

def main(train_cfg: dict, general_cfg: dict):
  import atexit
  def at_exit():
    net.save(general_cfg.get("output_path"))
    torch.save(net.state_dict(), "checkpoint.pt")
  atexit.register(at_exit)

  net = NN()
  if path.exists("checkpoint.pt"):
    net.load_state_dict(torch.load("checkpoint.pt"))
    print("Loaded network from checkpoint.")

  net = net.cuda().float()

  total = 0.0
  data = Set()
  loader = DataLoader(data, batch_size=train_cfg.get("batch_size"), pin_memory=True, num_workers=general_cfg.get("workers"))
  for e in range(train_cfg.get("epochs")):
    for i, (x, y) in enumerate(loader):
      total += net.train_step(x.cuda(), y.cuda())

      if i % train_cfg.get("report_freq") == train_cfg.get("report_freq") - 1:
        print(f"step {i + 1} loss {total / (i+1)}")
        wandb.log({"loss": total / (i+1)})

    if e % train_cfg.get("save_freq") == train_cfg.get("save_freq") - 1:
      net.save("checkpoint.rs")
      torch.save(net.state_dict(), "checkpoint.pt")
      print("Saved weights and checkpoint.")

    print(f"epoch {e + 1} loss {total / (i+1)}")
    total = 0.0
  sys_exit()

if __name__ == "__main__":
  from toml import load
  config = load(argv[1])
  main(config.get("training"), config)
