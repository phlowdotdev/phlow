# Environment Settings

Below is a list of **all** environment variables used by the application, combining those defined in both files, along with their descriptions, default values, and types.

## Environment Variables Table

| Variable                                        | Description                                                                                                                       | Default Value | Type    |
|-------------------------------------------------|-----------------------------------------------------------------------------------------------------------------------------------|--------------|---------|
| **PHLOW_PACKAGE_CONSUMERS_COUNT**               | **Number of package consumers**<br>Defines how many threads will be used to process packages.                                     | `10`         | `i32`   |
| **PHLOW_MIN_ALLOCATED_MEMORY_MB**               | **Minimum allocated memory (MB)**<br>Defines the minimum amount of memory, in MB, allocated to the process.                       | `10`         | `usize` |
| **PHLOW_GARBAGE_COLLECTION_ENABLED**            | **Enable garbage collection**<br>Enables or disables garbage collection (GC).                                                     | `true`       | `bool`  |
| **PHLOW_GARBAGE_COLLECTION_INTERVAL_SECONDS**   | **Garbage collection interval (seconds)**<br>Defines the interval at which garbage collection will be performed.                  | `60`         | `u64`   |
| **PHLOW_LOG**                                   | **Log level**<br>Defines the log verbosity for standard logging output. Possible values typically include `TRACE`, `DEBUG`, `INFO`, `WARN`, `ERROR`. | `WARN`        | `str`   |
| **PHLOW_SPAN**                                  | **Span level**<br>Defines the verbosity level for span (OpenTelemetry) tracing. Possible values typically include `TRACE`, `DEBUG`, `INFO`, `WARN`, `ERROR`. | `INFO`        | `str`   |
| **PHLOW_OTEL**                                  | **Enable OpenTelemetry**<br>Enables or disables OpenTelemetry tracing and metrics.                                                | `true`       | `bool`  |

---

## Notes

- If an environment variable is not set, the default value indicated in the table above will be used.
- Set the corresponding environment variables before running the application to override the defaults.
- The **log level** (`PHLOW_LOG`) and **span level** (`PHLOW_SPAN`) control different layers of logging:
  - `PHLOW_LOG`: Affects standard logging (e.g., error, warning, info messages).
  - `PHLOW_SPAN`: Affects tracing spans (useful for deeper telemetry insights with OpenTelemetry).
- The `PHLOW_OTEL` variable controls whether or not OpenTelemetry providers (for both tracing and metrics) are initialized.
