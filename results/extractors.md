# Extractors

Medians, 100 samples.

| Case | Median | Mean |
| ---- | ------ | ---- |
| micro JSON small | 1.75 µs | 2.25 µs |
| micro JSON 10 KB | 163.7 µs | 360 µs |
| micro query | 1.38 µs | 1.81 µs |
| micro cookies (5) | 3.90 µs | 20.8 µs |
| integrated JSON | 3.97 µs | 6.44 µs |
| integrated query | 3.50 µs | 4.33 µs |
| integrated path | 4.04 µs | 35.7 µs |
| integrated state | 2.79 µs | 4.11 µs |

![extractor cost](img/extractors.png)
