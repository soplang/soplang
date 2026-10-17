#!/usr/bin/env python3
"""Plot the comparison's median process times and interquartile ranges."""

import argparse
import json
from pathlib import Path

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt
from matplotlib.ticker import FuncFormatter


def main():
    here = Path(__file__).resolve().parent
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--input", type=Path, default=here / "results.json")
    parser.add_argument("--output", type=Path, default=here / "results.svg")
    parser.add_argument("--png", type=Path, help="Also export a PNG")
    args = parser.parse_args()
    data = json.loads(args.input.read_text())
    rows = data["workloads"]
    engines = [
        ("python", f"Python {data['python']['version'].split()[0]}", "#3776ab"),
        ("soplang2", "Soplang 2 · interpreter", "#ca6702"),
        ("soplang3_jit", "Soplang 3 · JIT", "#1b8872"),
        ("soplang3_llvm", "Soplang 3 · LLVM", "#7553a2"),
    ]
    plt.rcParams.update({"font.family": "DejaVu Sans", "svg.fonttype": "none", "svg.hashsalt": "soplang-comparison"})
    fig, ax = plt.subplots(figsize=(13, 10))
    width = 0.18
    all_medians = []
    for index, (engine, label, color) in enumerate(engines):
        stats = [row["engines"][engine] for row in rows]
        values = [stat["median_ns"] / 1e6 for stat in stats]
        all_medians += values
        errors = [[(stat["median_ns"] - stat["p25_ns"]) / 1e6 for stat in stats],
                  [(stat["p75_ns"] - stat["median_ns"]) / 1e6 for stat in stats]]
        positions = [row_index + (index - 1.5) * width for row_index in range(len(rows))]
        ax.barh(positions, values, height=width * 0.86, color=color, label=label,
                xerr=errors, error_kw={"capsize": 2, "elinewidth": 0.8, "ecolor": "#252525"})
        for position, value, stat in zip(positions, values, stats):
            label_x = max(value, stat["p75_ns"] / 1e6) * 1.08
            ax.text(label_x, position, f"{value:.2f}", va="center", fontsize=7.5, color=color)
    ax.set_xscale("log")
    ax.set_xlim(min(all_medians) / 1.8, max(all_medians) * 3)
    ax.xaxis.set_major_formatter(FuncFormatter(lambda value, _: f"{value:g}"))
    ax.set_yticks(range(len(rows)), [row["name"].replace("_", " ") for row in rows])
    ax.invert_yaxis()
    ax.set_xlabel("Median process wall time (milliseconds, logarithmic scale) · lower is faster", labelpad=12)
    ax.grid(axis="x", which="major", alpha=0.22)
    ax.set_axisbelow(True)
    for spine in ("top", "right", "left"):
        ax.spines[spine].set_visible(False)
    ax.tick_params(axis="y", length=0, labelsize=10)
    ax.legend(loc="lower left", bbox_to_anchor=(-0.01, 1.015), ncol=2, frameon=False)
    fig.suptitle("Python vs Soplang 2 vs Soplang 3", x=0.06, y=0.98, ha="left", fontsize=21, fontweight="bold")
    config = data["configuration"]
    fig.text(0.06, 0.94, f"{config['samples']} fresh processes per cell · median and IQR · every output validated", fontsize=11)
    fig.text(0.06, 0.04, data["machine"]["cpu"] + " · " + data["machine"]["platform"], fontsize=8.5)
    fig.text(0.06, 0.02, "Includes process startup; JIT includes compilation; LLVM build time is separate. One-workstation results.", fontsize=8.5)
    fig.subplots_adjust(left=0.20, right=0.96, top=0.85, bottom=0.11)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    fig.savefig(args.output, metadata={"Date": data["started_at"], "Creator": "Soplang benchmark suite"})
    if args.png:
        args.png.parent.mkdir(parents=True, exist_ok=True)
        fig.savefig(args.png, dpi=180)
    plt.close(fig)
    print(f"Wrote {args.output}; matplotlib {matplotlib.__version__}")


if __name__ == "__main__":
    main()
