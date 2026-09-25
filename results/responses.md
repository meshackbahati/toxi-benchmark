# Responses

Medians, 100 samples.

| Case | Median | Mean |
| ---- | ------ | ---- |
| JSON small struct | 0.71 µs | 0.73 µs |
| JSON medium value | 1.81 µs | 1.89 µs |
| JSON 1000 rows | 552.5 µs | 2.32 ms |
| text 1 KB | 0.58 µs | 0.63 µs |
| HTML page | 0.62 µs | 0.68 µs |
| empty ok | 0.40 µs | 1.21 µs |
| error to 404 | 2.37 µs | 3.16 µs |

![response construction](img/responses.png)
