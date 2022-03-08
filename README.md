# tracing-perf - Performance and time reporting for [tracing]

This crate provides very useful tools for reporting performance metrics
through `tracing`, similar to [slog-perf].

[tracing]: https://github.com/tokio-rs/tracing
[slog-perf]: https://github.com/slog-rs/perf

## Sample output

```
2022-02-28 00:14:37.197  INFO time-report: name: File Reader Input, path: 0.004322477, open: 0.138763265, send: 0.607652069
2022-02-28 00:14:37.258  INFO time-report: name: File Reader Output, recv: 0.039922656, read: 0.726326027, send: 0.044767856
2022-02-28 00:14:37.276  INFO time-report: name: Chunker, recv: 0.125705828, chunk: 0.346698769, send: 0.348204863
2022-02-28 00:14:37.277  INFO time-report: name: Chunk Processor, recv: 0.119203928, header: 0.040062816, check: 0.000777103, pack: 0.670504889, write: 0.000077240, send: 0.000154348
2022-02-28 00:14:37.277  INFO time-report: name: Chunk Processor, recv: 0.144801270, header: 0.040572927, check: 0.000959837, pack: 0.644118678, write: 0.000094198, send: 0.000525979
2022-02-28 00:14:37.279  INFO time-report: name: Chunk Processor, recv: 0.167694716, header: 0.039238876, check: 0.001081070, pack: 0.624880001, write: 0.000087065, send: 0.000268927
2022-02-28 00:14:37.282  INFO time-report: name: Chunk Processor, recv: 0.133540186, header: 0.044593754, check: 0.001045541, pack: 0.656200887, write: 0.000088400, send: 0.000389250
2022-02-28 00:14:37.282  INFO time-report: name: Chunk Processor, recv: 0.127383362, header: 0.041043529, check: 0.000909542, pack: 0.666432821, write: 0.000097267, send: 0.000490198
2022-02-28 00:14:37.295  INFO time-report: name: Chunk Processor, recv: 0.129950678, header: 0.042644522, check: 0.002428830, pack: 0.674105872, write: 0.000092808, send: 0.000155586
2022-02-28 00:14:37.296  INFO time-report: name: Entry Processor, recv entry: 0.009082137, process entry: 0.011987756, recv hash: 0.828862962
2022-02-28 00:14:37.296  INFO time-report: name: Async IO, recv: 0.830499883, read-metadata: 0.000270457, read-metadata send response: 0.000091742
2022-02-28 00:14:37.296  INFO time-report: name: Async IO, recv: 0.813757364, read: 0.000007788, read send response: 0.000001189, read-metadata: 0.000267161, read-metadata send response: 0.000091131
2022-02-28 00:14:37.296  INFO time-report: name: Async IO, recv: 0.823099455, read-metadata: 0.000274352, read-metadata send response: 0.000072718
2022-02-28 00:14:37.296  INFO time-report: name: Async IO, recv: 0.816578158, read-metadata: 0.000280856, read-metadata send response: 0.000089184
2022-02-28 00:14:37.296  INFO time-report: name: Async IO, recv: 0.808667088, read-metadata: 0.000256612, read-metadata send response: 0.000087649
2022-02-28 00:14:37.296  INFO time-report: name: Async IO, recv: 0.815601857, read-metadata: 0.000280478, read-metadata send response: 0.000091562
```