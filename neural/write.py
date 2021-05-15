import numpy as np

# the protocol is:
# (how_many_weights_per_neuron, flattened_weights, 8.36, biases, 8.366) for each layer
weights = np.random.random(size=(25))
weights = weights.tolist()
biases = np.random.random(size=(5))
biases = biases.tolist()

a = np.array([5]).flatten()
a = np.append(a, weights)
a=np.append(a, 8.36)
a=np.append(a, biases)
a=np.append(a, 8.366)
print(a.flatten())
np.save("plain.npy", a.flatten())