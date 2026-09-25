# Throughput

Medians, 100 samples. Concurrent dispatch fan-out, no TCP.

| Case | Batch median | Derived req/s |
| ---- | ------------ | ------------- |
| sequential text (1 req) | 2.07 µs | ~484,000 |
| sequential JSON (1 req) | 3.37 µs | ~297,000 |
| 4 tasks × 50 (200 reqs) | 953 µs | ~210,000 |
| 16 tasks × 50 (800 reqs) | 4.10 ms | ~195,000 |
| 50 tasks × 50 (2500 reqs) | 10.59 ms | ~236,000 |
| echo 16 × 25 (400 reqs) | 5.95 ms | ~67,000 |

![in-process throughput](img/throughput.png)
