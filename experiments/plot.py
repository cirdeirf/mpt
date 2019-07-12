import re
import matplotlib.pyplot as plt
import matplotlib.ticker as ticker
import numpy as np
from scipy.ndimage.filters import gaussian_filter1d

test_set = "0"
rk = True  # insertions / average rank
pr = False  # insertions / inverse probability
bp = False  # best parse relative error
if rk + pr + bp != 1:
    raise Exception("Only one plot at a time")

fig, ax = plt.subplots()
plt.gcf().set_size_inches(11.95, 6.85)
#  ax.grid(True)

with open("test" + test_set + "_mpt.log") as f_mpt:
    data_mpt = f_mpt.read()
data_mpt = [entry.split("\n") for entry in data_mpt.split("\n\n")]
with open("test" + test_set + "_best_parse.log") as f_bp:
    data_bp = f_bp.read()
data_bp = [entry.split("\n") for entry in data_bp.split("\n\n")]
data = zip(data_mpt, data_bp)

x_scatter = []
y_scatter = []
count_non_aborted = 0
count_same_tree = 0
count_same_pr = 0
for (entry_mpt, entry_bp) in data:
    insertions = 2e+7
    probability_mpt = 0.0
    average_rank = 0.0
    for line in entry_mpt:
        # mpt
        if line.startswith("mpt:"):
            mpt = line.split(" ", 1)[1]
        # probability
        elif line.startswith("probability:"):
            probability_mpt = float(line.split(" ")[1])
        # insertions
        elif line.startswith("insertions:"):
            insertions = int(line.split(" ")[1])
        # time
        elif line.startswith("time:"):
            time_str = line.split(" ")[1]
            if time_str.endswith("µs"):
                time = float(time_str[:-2])/1e+6
            elif time_str.endswith("ms"):
                time = float(time_str[:-2])/1e+3
            elif time_str.endswith("s"):
                time = float(time_str[:-1])
        # comments (extract average rank)
        elif line.startswith("%"):
            average_rank = float(re.sub(r"%|\(.*\)", "", line).split("/")[3])
    if rk:
        x_scatter.append(average_rank)
        y_scatter.append(np.log10(insertions))
    elif pr and probability_mpt != 0.0:
        x_scatter.append(1/probability_mpt)
        y_scatter.append(insertions)
    elif bp and probability_mpt != 0.0:
        count_non_aborted += 1
        for line in entry_bp:
            # best parse
            if line.startswith("best parse:"):
                best_parse = line.split(" ", 2)[2]
            elif line.startswith("probability:"):
                probability_bp = float(line.split(" ")[1])
        if mpt == best_parse:
            count_same_tree += 1
        if probability_mpt == probability_bp:
            count_same_pr += 1
        x_scatter.append(np.log10(probability_mpt))
        y_scatter.append((probability_mpt - probability_bp) / probability_mpt)

if bp:
    print("instances (< 20⁷): \t%d" % count_non_aborted)
    print("same output trees: \t%d (%.3f%%)" % (count_same_tree,
                                                100 * count_same_tree /
                                                count_non_aborted))
    print("same probability: \t%d (%.3f%%)" % (count_same_pr,
                                               100 * count_same_pr /
                                               count_non_aborted))

label_scatter = "automata"
if bp:
    label_scatter = r"$\frac{\widehat{p} - \widetilde{p}}{\widehat{p}}$"
ax.scatter(x_scatter, y_scatter, c="tab:blue", s=5, label=label_scatter)

# average rank to insertions
if rk:
    filename = "plot" + test_set + "_average_rank.pgf"
    x, y = zip(*sorted((xVal, np.mean([yVal for a, yVal in zip(x_scatter,
            y_scatter) if xVal==a])) for xVal in set(x_scatter)))

    poly = np.polyfit(x, y, 1)
    f = np.poly1d(poly)
    xp = np.linspace(x[0], x[-1], 500)
    yp = f(xp)
    ysmoothed = gaussian_filter1d(y, sigma=5)
    plt.plot(xp, yp, c="tab:red", label="linear approximation")
    #  plt.plot(x, y, "o", c="tab:green", alpha=0.5, label="arithmetic mean")

    ax.yaxis.set_major_formatter(ticker.FuncFormatter(lambda y, pos:
                                                      "$10^{%g}$" % y))
    plt.xlabel("average transition rank")
    plt.ylabel("insertions")

# inverse probability to insertions
elif pr:
    filename = "plot" + test_set + "_inverse_probability.pgf"
    x = np.linspace(min(x_scatter), max(x_scatter), 100)

    y = 4*x
    plt.plot(x, y, label=r"$\frac{1}{2\widehat{p}}$", c="tab:red")

    y = pow(x, 2)
    plt.plot(x, y, label=r"$\frac{1}{\widehat{p}^2}$", c="tab:red",
             linestyle="--")

    plt.xscale("log")
    plt.yscale("log")
    plt.xlabel(r"$\frac{1}{\widehat{p}}$")
    plt.ylabel("insertions")

# mpt probability to relative error of best parse probability
elif bp:
    filename = "plot" + test_set + "_bp_relative_error.pgf"

    ax.xaxis.set_major_formatter(ticker.FuncFormatter(lambda x, pos:
                                                      "$10^{%g}$" % x))
    ax.yaxis.set_major_formatter(ticker.PercentFormatter(1.0))
    plt.xlabel("most probable tree probability")
    plt.ylabel("relative error")

ax.legend()

plt.savefig(filename, bbox_inches="tight")
#  plt.show()
