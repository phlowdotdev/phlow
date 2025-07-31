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

## Local Module Paths

Starting from recent versions, Phlow supports loading modules from local paths instead of downloading them from remote repositories. This is particularly useful for:

- **Development**: Testing modules during development without publishing
- **Custom modules**: Using private or organization-specific modules
- **Debugging**: Working with local modifications of existing modules

### Path Formats

Phlow automatically detects local paths based on these prefixes:

- `./` - Relative path from current directory
- `../` - Relative path to parent directory
- `/` - Absolute path

### Examples

```yaml
modules:
  # Local module in current directory
  - module: ./my_custom_module
    with:
      config: "development"

  # Local module in parent directory
  - module: ../shared_modules/auth
    with:
      secret_key: !phs envs.AUTH_SECRET

  # Absolute path
  - module: /opt/phlow/modules/custom_logger
    with:
      log_level: "debug"

  # Regular remote module (for comparison)
  - module: postgres
    version: latest
    with:
      host: localhost
```

### Local Module Structure

Local modules must follow the same structure as remote modules:

```
my_custom_module/
├── module.so          # Compiled module binary
└── phlow.yaml         # Module metadata (optional)
```

The `phlow.yaml` file should contain module information:

```yaml
module: my_custom_module
version: "1.0.0"
input:
  type: object
  properties:
    data:
      type: string
output:
  type: string
```

### Benefits

- **No Download**: Local modules skip the download process, improving startup time
- **Development Workflow**: Test changes immediately without publishing
- **Version Control**: Keep custom modules alongside your project
- **Offline Support**: Work without internet connectivity

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
