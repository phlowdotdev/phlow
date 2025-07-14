---
sidebar_position: 3
title: Modules (modules.yaml)
---
Modules define components or services needed by the flow.

## Structure

| Field | Description |
|-------|-------------|
| `module` | The module's name. |
|| `version` | **Optional.** The module's version. If not specified, defaults to `latest`. |
| `with` | Configuration for the module. |

Example CLI module definition:

```yaml
- module: cli
  version: latest
  with:
    additional_args: false
    args: 
      - name: name
        description: Student name
        index: 1
        type: string
        required: true
      - name: age
        description: Student age
        index: 2
        type: number
        required: true
      - name: force
        long: force
        description: Force assertion
        short: f
        type: boolean
        default: false
```

## With scripts

Each module can accept any property within `with`. It is also possible to execute `!phs` within `with` to run scripts or declare environment variables, as shown in the example:

```yaml
- module: postgres
  version: latest
  with:
    host: !phs envs.POSTGRES_HOST ?? 'localhost'
    user: !phs envs.POSTGRES_USER ?? 'postgres'
    password: !phs envs.POSTGRES_PASSWORD
```

## Optional Version

Starting from recent versions, the `version` field is optional. If not specified, Phlow will automatically use `latest`:

```yaml
# Both declarations are equivalent
- module: cli
  # version omitted - will use 'latest'
  with:
    additional_args: false
    args: 
      - name: name
        description: Student name
        index: 1
        type: string
        required: true

- module: postgres
  version: latest  # explicitly specified
  with:
    host: localhost
    user: postgres
```

This simplifies module declarations while maintaining backward compatibility.

## Good practice

A good practice is to keep the modules in a separate file, such as `modules.yaml`, and reference them in the `main.phlow` using `!include modules.yaml`. This helps maintain organized and easily maintainable code.

#### Example of `modules.yaml`:

```yaml
- module: cli
  version: latest
  with:
    additional_args: false
    args: 
      - name: name
        description: Student name
        index: 1
        type: string
        required: true
      - name: age
        description: Student age
        index: 2
        type: number
        required: true
      - name: force
        long: force
        description: Force assertion
        short: f
        type: boolean
        default: false
- module: postgres
  version: latest
  with:
    host: !phs envs.POSTGRES_HOST ?? 'localhost'
    user: !phs envs.POSTGRES_USER ?? 'postgres'
    password: !phs envs.POSTGRES_PASSWORD
```

#### Example of `main.phlow` referencing `modules.yaml`:

```yaml
modules: !include modules.yaml
steps:
  - step: initialize
    description: Initialize the environment
  - step: run-cli
    module: cli
    with:
      args:
        - name: input
          description: Input data
          index: 1
          type: string
          required: true
  - step: setup-database
    module: postgres
    with:
      database: my_database
```
