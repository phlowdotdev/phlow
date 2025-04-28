---
sidebar_position: 9
title: Environment Variables
---
# Environment Settings

Below is a list of all environment variables used by the application, combining those defined in both files, along with their descriptions, default values, and types.

## Environment Variables Table

| Variable  | Description  Default Value | Type    |
|-----------|----------------------------|---------|
| PHLOW_PACKAGE_CONSUMERS_COUNT               | Number of package consumers Defines how many threads will be used to process packages.                                     | `10`         | `i32`   |
| PHLOW_MIN_ALLOCATED_MEMORY_MB               | Minimum allocated memory (MB) Defines the minimum amount of memory, in MB, allocated to the process.                       | `10`         | `usize` |
| PHLOW_GARBAGE_COLLECTION_ENABLED            | Enable garbage collection Enables or disables garbage collection (GC).                                                     | `true`       | `bool`  |
| PHLOW_GARBAGE_COLLECTION_INTERVAL_SECONDS   | Garbage collection interval (seconds) Defines the interval at which garbage collection will be performed.                  | `60`         | `u64`   |
| PHLOW_LOG                                   | Log level Defines the log verbosity for standard logging output. Possible values typically include `TRACE`, `DEBUG`, `INFO`, `WARN`, `ERROR`. | `WARN`        | `str`   |
| PHLOW_SPAN                                  | Span level Defines the verbosity level for span (OpenTelemetry) tracing. Possible values typically include `TRACE`, `DEBUG`, `INFO`, `WARN`, `ERROR`. | `INFO`        | `str`   |
| PHLOW_OTEL                                  | Enable OpenTelemetry Enables or disables OpenTelemetry tracing and metrics.                                                | `true`       | `bool`  |


## Additional Notes
> - The default values are provided for each variable, and they can be overridden by setting the corresponding environment variable.
> - The `PHLOW_PACKAGE_CONSUMERS_COUNT` variable controls the number of threads used for processing packages, which can be adjusted based on the workload.
> - If an environment variable is not set, the default value indicated in the table above will be used.
> - Set the corresponding environment variables before running the application to override the defaults.
> - The log level (`PHLOW_LOG`) and span level (`PHLOW_SPAN`) control different layers of logging:
>   - `PHLOW_LOG`: Affects standard logging (e.g., error, warning, info messages).
>   - `PHLOW_SPAN`: Affects tracing spans (useful for deeper telemetry insights with OpenTelemetry).
> - The `PHLOW_OTEL` variable controls whether or not OpenTelemetry providers (for both tracing and metrics) are initialized.
