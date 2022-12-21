# Logging and Monitoring

For more information on each side, see:
* [Monitoring in Java](monitoring-in-java.md)
* [Monitoring in Rust](monitoring-in-rust.md)

On the logging front, we have separate logs on the Rust and Java side.

For observability, we use Prometheus. There is support for [Prometheus metrics](https://prometheus.io/docs/concepts/metric_types/) on both the Java and the Rust sides.

Both sets of Prometheus metric values are passed through and exposed by the Prometheus API in Java.
This is configured on its own port (by default, port `3335`).
