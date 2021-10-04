import torch
from torch.functional import Tensor
import torch.optim as optim
import torch.nn as nn
from torch.utils.data import DataLoader, IterableDataset
import torch.nn.functional as F
from chess import BLACK, BaseBoard, SQUARES
from math import ceil
from numpy import array

def cross_entropy_loss(x, p):
  return torch.mean(-(p*F.logsigmoid(x) + (1-p)*F.logsigmoid(-x)))

class Set(IterableDataset):
  def __init__(self) -> None:
    super(Set, self).__init__()

  def sample_iter(self) -> None:
    worker_info = torch.utils.data.get_worker_info()
    worker_id = worker_info.id

    f = open(f"./data/run3/{worker_id+8}.txt", "r").readlines()
    per_worker = int(ceil(len(f) / float(worker_info.num_workers)))
    iter_start = worker_id * per_worker
    iter_end = min(iter_start + per_worker, len(f))
    for i in range(iter_start, iter_end):
      a = f[i].split("|")
      yield self.build(a[0][:a[0].find(" ")], "w" in a[0]), array((int(a[1]) + 0.5 * int(a[2])) / 1024)

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

    self.optimizer = optim.Adadelta(self.parameters(), 0.01)

  def forward(self, x) -> Tensor:
    x = F.relu(self.fc0(x))
    x = F.relu(self.fc1(x))
    x = F.relu(self.fc2(x))
    x = torch.sigmoid(self.fc3(x))
    return x

  def train_step(self, inputs, y) -> float:
    self.optimizer.zero_grad()

    out = self(inputs.float())
    loss = cross_entropy_loss(out, y.float())
    loss.backward()

    self.optimizer.step()

    return loss

  def save(self) -> None:
    f = open("net.rs", "a+")
    for (name, param) in self.named_parameters():
      name = name.replace(".", "_")
      try:
        f.write(f"pub const {name.upper()}: [[f32; {len(param[0])}]; {len(param)}] = {param.tolist()};\n")
      except:
        f.write(f"pub const {name.upper()}: [f32; {len(param)}] = {param.tolist()};\n")

def main():
  net = NN().float().cuda()

  data = Set()
  loader = DataLoader(data, batch_size=1024, pin_memory=True, num_workers=4)
  total = 0.0
  for i, (x, y) in enumerate(loader):
    total += net.train_step(x.cuda(), y.cuda())
    if i % 1000 == 999:
      print(f"step {i + 1} loss {total / 1000}")
      total = 0.0
  net.save()

if __name__ == "__main__":
  main()