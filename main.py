import json

with open("data.json", "r") as f:
  data = json.load(f)


import matplotlib.pyplot as plt

plt.hist(data, bins=list(range(1000)))
# plt.xlim(0, 500)

plt.title("Game Lengths")
plt.xlabel("# Turns")
plt.ylabel("Frequency")

plt.grid(axis="y", alpha=0.75)
plt.show()
