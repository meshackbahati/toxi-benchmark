# Extractors

Medians. Loaded runs: 100 samples. Quiet box: 50+ samples, load near 1.

| Case | Loaded | Quiet box |
| ---- | ------ | --------- |
| micro JSON small | 1.75 µs | 0.78 µs |
| micro JSON 10 KB | 163.7 µs | 48.7 µs |
| micro query | 1.38 µs | 0.57 µs |
| micro cookies (5) | 3.90 µs | 1.61 µs |
| integrated JSON | 3.97 µs | 1.25 µs |
| integrated query | 3.50 µs | 1.28 µs |
| integrated path | 4.04 µs | 1.34 µs |
| integrated state | 2.79 µs | 1.07 µs |

![extractor cost](img/extractors.png)
