# Chart theme for benchmarks/img (gnuplot).
# regenerate: gnuplot benchmarks/img/theme.gp  (reads /tmp/opencode/*.dat)
# Base style shared by every chart: boxed legend with title, y grid,
# value labels on bars, consistent framework palette.

# palette: toxi teal, poem amber, salvo sky, warp violet, loco slate, rocket rose
TOXI="#0d9488"
POEM="#d97706"
SALVO="#0284c7"
WARP="#7c3aed"
LOCO="#475569"
ROCKET="#e11d48"

set terminal pngcairo size 900,500 font ",11" enhanced
set style fill solid 0.9 border -1
set grid ytics lt 0 lc rgb "#e2e8f0"
set border lc rgb "#94a3b8"
set key outside top center horizontal box lt 1 lc rgb "#94a3b8" title "framework" font ",10"
set tics font ",10"
set title font ",13"
