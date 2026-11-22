#!/usr/bin/env bash
# Measures the release binary: time until the DB is ready (startup proxy), idle CPU% and PSS memory
# (app + WebKit child processes). Uses scratch XDG dirs so your real data is untouched.
# usage: scripts/measure-perf.sh <scratch-dir> <path-to-ps4-studio-binary>
set -u
S=$1
BIN=$2
export XDG_DATA_HOME="$S/xdg/data" XDG_CONFIG_HOME="$S/xdg/config" XDG_CACHE_HOME="$S/xdg/cache"
rm -rf "$S/xdg"
mkdir -p "$XDG_DATA_HOME"
t0=$(date +%s.%N)
"$BIN" >/dev/null 2>&1 &
pid=$!
while [ ! -f "$XDG_DATA_HOME/ps4-studio/ps4-studio.db" ]; do
	sleep 0.01
	kill -0 "$pid" 2>/dev/null || {
		echo "app exited early"
		exit 1
	}
done
t1=$(date +%s.%N)
echo "db_ready_s=$(echo "$t1 - $t0" | bc)"
sleep 8 # let the webview settle

tree() {
	local p=$1
	echo "$p"
	for c in $(pgrep -P "$p"); do tree "$c"; done
}
pids=$(tree "$pid")
ticks() {
	local t=0 p
	for p in $pids; do
		[ -r "/proc/$p/stat" ] && t=$((t + $(awk '{print $14+$15}' "/proc/$p/stat")))
	done
	echo "$t"
}
pss() {
	local t=0 p
	for p in $pids; do
		[ -r "/proc/$p/smaps_rollup" ] && t=$((t + $(awk '/^Pss:/{print $2}' "/proc/$p/smaps_rollup")))
	done
	echo "$t"
}
a=$(ticks)
sleep 10
b=$(ticks)
hz=$(getconf CLK_TCK)
echo "procs=$(echo "$pids" | wc -w)"
echo "idle_cpu_pct_over_10s=$(echo "scale=2; ($b - $a) / $hz / 10 * 100" | bc)"
echo "pss_total_mb=$(echo "scale=1; $(pss) / 1024" | bc)"
kill "$pid" 2>/dev/null
sleep 1
for p in $pids; do kill "$p" 2>/dev/null; done
exit 0
