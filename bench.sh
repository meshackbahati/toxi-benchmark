#!/usr/bin/env bash
# Comprehensive benchmark runner: matrix A-P across all six frameworks.
# Usage: ./bench.sh [--quick]   (--quick: single run, shorter durations)
# Output: benchmarks/raw/<fw>/, benchmarks/results/*.csv, environment in
# benchmarks/environment.txt (repo copy updated at the end).
set -u
export PATH="$HOME/.cargo/bin:$HOME/.config/opencode/bin:$PATH"

KIT="$(cd "$(dirname "$0")" && pwd)"
REPO=/home/bealthguy/.projects/programming/crates/toxi
RAW="$REPO/benchmarks/raw"
RES="$REPO/benchmarks/results"
BODY=/tmp/opencode/matrix-body.json
QUICK=0
[ "${1:-}" = "--quick" ] && QUICK=1

DUR=15s; WARM=3s
if [ "$QUICK" = 0 ]; then
  : # durations below are the full-matrix defaults
fi

mkdir -p "$RAW" "$RES"
cd "$KIT"
# Clear stray servers from interrupted runs so the port is free.
pkill -f 'target/release/.*-server' 2>/dev/null || true
sleep 2
echo '{"name":"Meshack","language":"rust","framework":"toxi"}' > "$BODY"
printf 'framework,framework_version,commit,benchmark,payload_size,connections,threads,duration_seconds,run,requests_per_second,p50_ms,p95_ms,p99_ms,errors,success_rate,rss_mb,timestamp\n' > "$RES/matrix.csv"

hwm() { grep VmHWM /proc/$1/status 2>/dev/null | awk '{print $2}'; }

oha_json() { # url [extra oha args...] -> stdout json
  oha -z "$DUR" -c "$CONNS" --output-format json "$@" 2>/dev/null
}

record() { # fw ver commit bench payload conns dur run jsonfile rss_kb
  python3 - "$1" "$2" "$3" "$4" "$5" "$6" "$7" "$8" "$9" "${10}" <<'EOF' >> "$RES/matrix.csv"
import json,sys,time
fw,ver,commit,bench,payload,conns,dur,run,jf,rss=[sys.argv[i] for i in range(1,11)]
d=json.load(open(jf)); s=d['summary']; lp=d['latencyPercentiles']
err=sum(v for k,v in d.get('errorDistribution',{}).items())
rss_mb=float(rss)/1024.0
print(','.join(map(str,[fw,ver,commit,bench,payload,conns,'auto',dur,run,
  round(s['requestsPerSec'],1), round(lp['p50']*1000,3), round(lp.get('p95',lp['p99'])*1000,3),
  round(lp['p99']*1000,3), err, s['successRate'], round(rss_mb,1),
  time.strftime('%Y-%m-%dT%H:%M:%SZ', time.gmtime())])))
EOF
}

start_fw() { # name -> sets SRV pid, waits readiness
  local fw=$1 port=8080
  if [ "$fw" = loco-server ]; then
    cd "$KIT/servers"
    ../target/release/loco-server start --port $port > /tmp/opencode/mx-$fw.log 2>&1 &
    SRV=$!
    cd "$KIT"
  else
    ./target/release/$fw > /tmp/opencode/mx-$fw.log 2>&1 &
    SRV=$!
  fi
  for _ in $(seq 1 60); do
    # Connection success (any status) means the server is up.
    ffcurl -s -o /dev/null http://127.0.0.1:$port/ > /dev/null 2>&1 && return 0
    sleep 0.25
  done
  echo "FAILED readiness: $fw"; return 1
}

stop_fw() { kill $SRV 2>/dev/null; sleep 2; }

measure() { # fw benchname method url [oha-extra...] — warmup then run, saves raw
  local fw=$1 bench=$2 method=$3 url=$4; shift 4
  local tag="${fw}-${bench}"
  oha -z "$WARM" -c "$CONNS" ${method:+-m "$method"} "$@" "$url" > /dev/null 2>&1
  if [ "$method" = POST ]; then
    oha -z "$DUR" -c "$CONNS" --output-format json -m POST \
      -H "content-type: application/json" -D "$BODY" "$@" "$url" > "$RAW/$fw-$bench.json" 2>/dev/null
  else
    oha -z "$DUR" -c "$CONNS" --output-format json "$@" "$url" > "$RAW/$fw-$bench.json" 2>/dev/null
  fi
}

TOXI_SHA=$(git -C "$REPO" rev-parse --short HEAD)
ver_of() { # framework crate -> version from kit lockfile
  grep -A2 "name = \"$1\"" "$KIT/servers/Cargo.lock" 2>/dev/null | grep version | head -1 | cut -d'"' -f2
}
fw_crate() {
  case $1 in
    toxi-server) echo toxi;; rocket-server) echo rocket;; poem-server) echo poem;;
    salvo-server) echo salvo;; warp-server) echo warp;; loco-server) echo loco-rs;;
  esac
}

run_fw() { # fw binary versions...
  local fw=$1 ver=$2
  start_fw "$fw" || return 1
  local peak_kb=0
  sample_rss() { local h; h=$(hwm $SRV); [ -n "$h" ] && [ "$h" -gt "$peak_kb" ] && peak_kb=$h; }
  local B=http://127.0.0.1:8080
  CONNS=50
  for cfg in "A-plaintext GET $B/|C-1k GET $B/payload/1024|C-10k GET $B/payload/10240|C-100k GET $B/payload/102400|C-1m GET $B/payload/1048576|D-param GET $B/users/123456|E-query GET $B/search?q=hello&page=10&limit=20|F-echo POST $B/echo|K-cpu GET $B/cpu|L-sleep GET $B/sleep|N-404 GET $B/missing|N-400 GET $B/bad"; do
    bench=${cfg%% *}; rest=${cfg#* }; method=${rest%% *}; url=${rest#* }
    payload=""; case $bench in C-*) payload=${bench#C-};; esac
    measure "$fw" "$bench" "$method" "$url"
    sample_rss
    record "$fw" "$ver" "$TOXI_SHA" "$bench" "$payload" "$CONNS" 15 "$RUNNO" "$RAW/$fw-$bench.json" "$peak_kb"
  done
  # G middleware overhead (MW=1 server)
  stop_fw
  MW=1 start_fw "$fw" || { start_fw "$fw" || return 1; }
  measure "$fw" "G-mw" "GET" "$B/json"
  sample_rss
  record "$fw" "$ver" "$TOXI_SHA" "G-mw" "" "$CONNS" 15 "$RUNNO" "$RAW/$fw-G-mw.json" "$peak_kb"
  stop_fw
  start_fw "$fw" || return 1
  # H concurrency sweep on /json (short runs)
  for c in 1 10 25 50 125 250 500 1000; do
    CONNS=$c
    measure "$fw" "H-c$c" "GET" "$B/json"
    sample_rss
    record "$fw" "$ver" "$TOXI_SHA" "H-c$c" "" "$c" 15 "$RUNNO" "$RAW/$fw-H-c$c.json" "$peak_kb"
  done
  # I keep-alive (oha default) and J churn
  CONNS=50
  measure "$fw" "I-keepalive" "GET" "$B/json"
  record "$fw" "$ver" "$TOXI_SHA" "I-keepalive" "" 50 15 "$RUNNO" "$RAW/$fw-I-keepalive.json" "$peak_kb"
  oha -z "$WARM" -c 50 --disable-keepalive "$B/json" > /dev/null 2>&1
  oha -z "$DUR" -c 50 --output-format json --disable-keepalive "$B/json" > "$RAW/$fw-J-churn.json" 2>/dev/null
  record "$fw" "$ver" "$TOXI_SHA" "J-churn" "" 50 15 "$RUNNO" "$RAW/$fw-J-churn.json" "$peak_kb"
  stop_fw
}

# B baseline x3 + P sustained per framework, then full matrix x1
for RUNNO in 1 2 3; do
  for fw in toxi-server rocket-server poem-server salvo-server warp-server loco-server; do
    start_fw "$fw" || continue
    B=http://127.0.0.1:8080; CONNS=125
    oha -z 10s -c 125 "$B/json" > /dev/null 2>&1
    oha -z 30s -c 125 --output-format json "$B/json" > "$RAW/$fw-B-r$RUNNO.json" 2>/dev/null
    record "$fw" "$(ver_of $(fw_crate $fw))" "$TOXI_SHA" "B-json" "" 125 30 "$RUNNO" "$RAW/$fw-B-r$RUNNO.json" "$(hwm $SRV)"
    stop_fw
    sleep 2
  done
done

RUNNO=1
for fw in toxi-server rocket-server poem-server salvo-server warp-server loco-server; do
  run_fw "$fw" "$(ver_of $(fw_crate $fw))"
done

# P sustained 5 min per framework
for fw in toxi-server rocket-server poem-server salvo-server warp-server loco-server; do
  start_fw "$fw" || continue
  oha -z 300s -c 50 --output-format json "http://127.0.0.1:8080/json" > "$RAW/$fw-P-sustained.json" 2>/dev/null
  record "$fw" "$(ver_of $(fw_crate $fw))" "$TOXI_SHA" "P-sustained" "" 50 300 1 "$RAW/$fw-P-sustained.json" "$(hwm $SRV)"
  stop_fw
  sleep 2
done

echo "matrix complete: $RES/matrix.csv"
