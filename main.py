import json
import matplotlib.pyplot as plt

bins = list(range(1000))

with open("honorable_war.json", "r") as f:
  plt.hist(json.load(f), bins=bins, density=True, label="Honorable War")

with open("standard_war.json", "r") as f:
  plt.hist(json.load(f), bins=bins, density=True, label="Standard War")

# plt.xlim(0, 500)

plt.title("Game Lengths")
plt.xlabel("# Turns")
plt.ylabel("Frequency")
plt.legend()

plt.grid(axis="y", alpha=0.75)
plt.show()
