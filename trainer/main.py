import torch
from torch.functional import Tensor
import torch.nn as nn
from torch.utils.data import DataLoader, IterableDataset
import torch.nn.functional as F
from torch import optim
from torch.optim.lr_scheduler import StepLR
from chess import BLACK, BaseBoard, SQUARES
from numpy import array
from random import randint
from sys import exit as sys_exit, argv
from os import path
import wandb
wandb.init("rust-engine")

def cross_entropy_loss(x, p):
  return torch.mean(-(p*F.logsigmoid(x) + (1-p)*F.logsigmoid(-x)))

class Set(IterableDataset):
  def __init__(self) -> None:
    super(Set, self).__init__()

  def sample_iter(self):
    worker_info = torch.utils.data.get_worker_info()
    with open("./data_old_d8_wdl.txt") as f:
      for _ in range(300_000_000//worker_info.num_workers):
        try:
          aa = randint(0, 300_000_000)
          a = f.readline(aa)[:-1].split("|")
          yield self.build(a[0][:a[0].find(" ")], not "w" in a[0]), array(float(a[1]) / 1024)
        except:
          continue

  def build(self, s, flip) -> list[list[int]]:
    a = [0]*64*12
    if flip:
      b = BaseBoard(s).mirror()
    else:
      b = BaseBoard(s)
    for i in SQUARES:
      p = b.piece_at(i)
      if p:
        if p.color == BLACK:
          a[((p.piece_type + 6) % 12) * 64 + i] = 1.0
        else:
          a[(p.piece_type - 1) * 64 + i] = 1.0
    return array([a])

  def __iter__(self):
    for sample in self.sample_iter():
      yield sample

class NN(nn.Module):
  def __init__(self) -> None:
    super(NN, self).__init__()
    self.fc0 = nn.Linear(768, 128)
    self.fc1 = nn.Linear(128, 1)

  def forward(self, x) -> Tensor:
    x = self.fc0(x)
    x = F.relu(x)
    x = self.fc1(x)
    return x

  def train_step(self, inputs: Tensor, y: Tensor, optimizer) -> float:
    out = self(inputs.float())
    loss = torch.pow(y.sigmoid() - out.sigmoid(), 2).mean()
    loss.backward()

    optimizer.step()
    optimizer.zero_grad()

    return loss.item()

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
  from adabelief_pytorch import AdaBelief
  optimizer = AdaBelief(net.parameters(), lr=1e-3, eps=1e-16, rectify=False, print_change_log=False)

  total = 0.0
  data = Set()
  print(data.build("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR", False))
  loader = DataLoader(data, batch_size=train_cfg.get("batch_size"), pin_memory=True, num_workers=general_cfg.get("workers"))
  for e in range(train_cfg.get("epochs")):
    for i, (x, y) in enumerate(loader):
      temp = net.train_step(x.cuda(), y.cuda(), optimizer)
      total += temp

      if i % train_cfg.get("report_freq") == train_cfg.get("report_freq") - 1:
        report_freq = train_cfg.get("report_freq")
        print(f"step {i + 1} loss {total / report_freq}")
        total = 0.0
      wandb.log({"loss": temp})

    if e % train_cfg.get("save_freq") == train_cfg.get("save_freq") - 1:
      net.save("checkpoint.rs")
      torch.save(net.state_dict(), "checkpoint.pt")
      print("Saved weights and checkpoint.")

    report_freq = train_cfg.get("report_freq")
    print(f"epoch {e + 1} loss {total / (i % report_freq)}")
    total = 0.0
  sys_exit()

if __name__ == "__main__":
  from toml import load
  config = load(argv[1])
  main(config.get("training"), config)
