---
sidebar_position: 2
title: Module Types
---

Phlow provides two types of modules: `main` and `step`.

- **Main Module**: Serves as the application's entry point, starting the app in different contexts such as HTTP, CLI, or AMQP.
- **Step Module**: Contains the logic executed within a flow, such as logging, data fetching, or transformations.

These modules enable flexibility and reusability, making it easier to build modular and scalable applications.

Step modules can also be executed directly from Phlow Script (PHS), making it easy to use simple modules inside .phs or .rhai files.

### Example: Step Module with Phlow Script (PHS)
#### main.phlow
```phlow
main: cli
name: Example Cli
version: 1.0.0
description: Example CLI module
author: Your Name
modules:
  - module: cli
    version: latest
    with:
      additional_args: false
      args:
        - name: name
          description: Name of the user
          index: 1
          type: string
          required: false
  - module: log
    version: latest
steps:
  - return: !import script.phs
```

#### script.phs
```rust
log("warn", `Hello, ${main.name}`);
"phs"
```

To execute this file, just run:
```bash
2025-04-23T05:23:25.474573Z  WARN log: Hello, Phlow!
phs
```

This will evaluate the imported .phs file and run the steps using the declared modules.

> ℹ️ **Note:** In Phlow Script (PHS), function calls respect the **order of parameters** defined in the module's package. For example, if your `phlow.yaml` for the `log` module defines inputs like:
>
> ```phlow
> input: 
>   type: object
>   required: true
>   properties:
>     level:
>       type: string
>       description: The log level (e.g., info, debug, warn, error).
>       default: info
>       required: false
>     message:
>       type: string
>       description: The message to log.
>       required: true
> ```
>
> Then the correct function signature in `.phs` is:
>
> ```phs
> log(level, message)
> ```
>
> because the parameter order defined in `properties` is preserved and required by the execution engine.

