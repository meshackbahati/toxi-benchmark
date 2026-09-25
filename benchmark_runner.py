#!/usr/bin/env python3
"""Six-framework HTTP shootout runner.

Builds all servers in release mode, then for each framework: starts the
server on port 8080, applies load with oha (125 connections, 30 s) against
GET /json, records requests-per-second with p99 latency, terminates the
server, and moves on. Renders a dual-axis horizontal bar chart to
benchmark_results.png and prints a markdown summary table.

Usage: python3 benchmark_runner.py [--conns 125] [--duration 30s]
"""

import argparse
import json
import os
import signal
import subprocess
import sys
import time

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt

ROOT = os.path.dirname(os.path.abspath(__file__))
SERVERS_DIR = os.path.join(ROOT, "servers")
BIN_DIR = os.path.join(ROOT, "target", "release")
PORT = 8080
URL = f"http://127.0.0.1:{PORT}/json"

# name -> (binary, extra args, working directory)
FRAMEWORKS = {
    "toxi": ("toxi-server", [], SERVERS_DIR),
    "rocket": ("rocket-server", [], SERVERS_DIR),
    "poem": ("poem-server", [], SERVERS_DIR),
    "salvo": ("salvo-server", [], SERVERS_DIR),
    "warp": ("warp-server", [], SERVERS_DIR),
    # loco reads config/development.yaml relative to cwd
    "loco": ("loco-server", ["start", "--port", str(PORT)], SERVERS_DIR),
}

OHA = os.path.expanduser("~/.cargo/bin/oha")


def run(cmd, **kwargs):
    return subprocess.run(cmd, capture_output=True, text=True, **kwargs)


def build_all():
    print("building release binaries...", flush=True)
    r = run(
        ["cargo", "build", "--release", "--manifest-path",
         os.path.join(SERVERS_DIR, "Cargo.toml")]
    )
    if r.returncode != 0:
        print(r.stderr[-4000:])
        sys.exit("release build failed")


def wait_ready(proc, timeout=30.0):
    import urllib.request

    deadline = time.time() + timeout
    while time.time() < deadline:
        if proc.poll() is not None:
            return False
        try:
            with urllib.request.urlopen(URL, timeout=1) as resp:
                if resp.status == 200:
                    return True
        except Exception:
            time.sleep(0.2)
    return False


def stop(proc):
    if proc.poll() is None:
        proc.send_signal(signal.SIGTERM)
        try:
            proc.wait(timeout=5)
        except subprocess.TimeoutExpired:
            proc.kill()


def load_one(name, binary, args, cwd, conns, duration):
    path = os.path.join(BIN_DIR, binary)
    log = open(os.path.join("/tmp", f"shootout-{name}.log"), "w")
    proc = subprocess.Popen(
        [path] + args, cwd=cwd, stdout=log, stderr=subprocess.STDOUT
    )
    try:
        if not wait_ready(proc):
            print(f"{name}: failed to become ready, skipping")
            return None
        # brief warmup outside measurement
        run([OHA, "-z", "3s", "-c", "16", "--output-format", "json", URL],
            check=False)
        r = run([OHA, "-z", duration, "-c", str(conns),
                 "--output-format", "json", URL])
        data = json.loads(r.stdout)
        summary = data["summary"]
        return {
            "rps": summary["requestsPerSec"],
            "p99": data["latencyPercentiles"]["p99"] * 1000.0,
            "success": summary["successRate"],
        }
    finally:
        stop(proc)
        log.close()


def chart(results):
    names = list(results)
    rps = [results[n]["rps"] for n in names]
    p99 = [results[n]["p99"] for n in names]
    order = sorted(range(len(names)), key=lambda i: rps[i])
    names = [names[i] for i in order]
    rps = [rps[i] for i in order]
    p99 = [p99[i] for i in order]

    fig, ax1 = plt.subplots(figsize=(10, 6))
    y = range(len(names))
    ax1.barh(y, rps, color="#10b981")
    ax1.set_yticks(list(y))
    ax1.set_yticklabels(names)
    ax1.set_xlabel("throughput (req/s)")
    ax1.set_title("GET /json — throughput with p99 latency")

    ax2 = ax1.twiny()
    ax2.plot(p99, y, color="#f59e0b", marker="o", linewidth=2)
    ax2.set_xlabel("p99 latency (ms)")

    fig.tight_layout()
    out = os.path.join(ROOT, "benchmark_results.png")
    fig.savefig(out, dpi=120)
    print(f"chart: {out}")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--conns", type=int, default=125)
    ap.add_argument("--duration", default="30s")
    args = ap.parse_args()

    build_all()
    results = {}
    for name, (binary, extra, cwd) in FRAMEWORKS.items():
        print(f"running {name}...", flush=True)
        m = load_one(name, binary, extra, cwd, args.conns, args.duration)
        if m is not None:
            results[name] = m
            print(f"  rps={m['rps']:.0f} p99={m['p99']:.2f}ms ok={m['success']:.3f}",
                  flush=True)
        time.sleep(2)

    print("\n| framework | req/s | p99 (ms) | success |")
    print("| --- | ---: | ---: | ---: |")
    for name, m in sorted(results.items(), key=lambda kv: -kv[1]["rps"]):
        print(f"| {name} | {m['rps']:.0f} | {m['p99']:.2f} | {m['success']:.3f} |")
    chart(results)


if __name__ == "__main__":
    main()
