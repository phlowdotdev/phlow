---
sidebar_position: 3
title: Modules (phlow.yaml)
---
Modules define components or services needed by the flow.

## Structure

| Field | Description |
|-------|-------------|
| `module` | The module's name. |
|| `version` | **Optional.** The module's version. If not specified, defaults to `latest`. |
| `with` | Configuration for the module. |

Example CLI module definition:

```phlow
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

```phlow
- module: postgres
  version: latest
  with:
    host: !phs envs.POSTGRES_HOST ?? 'localhost'
    user: !phs envs.POSTGRES_USER ?? 'postgres'
    password: !phs envs.POSTGRES_PASSWORD
```

## Optional Version

Starting from recent versions, the `version` field is optional. If not specified, Phlow will automatically use `latest`:

```phlow
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

```phlow
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

Local modules can be either **compiled Rust modules** or **Phlow modules**:

#### Compiled Rust Modules
Traditional modules follow this structure:

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

#### Phlow Modules (.phlow files)

**New Feature**: Phlow now supports modules written entirely in Phlow format. These are `.phlow` files that define reusable logic without requiring Rust compilation.

```
route.phlow            # Single file module
```

A Phlow module consists of four main sections:

```phlow
# Configuration schema - defines what parameters the module accepts
with:
  type: object
  required: true
  properties:
    path:
      type: string
      required: true
    method:
      type: enum
      enum: [GET, POST, DELETE, PUT, PATCH, OPTIONS]
      required: true
    default_response:
      type: object
      required: false

# Input schema - defines the structure of runtime input data
input:
  type: object
  required: true
  properties:
    request:
      type: object
      properties:
        path: { type: string, required: true }
        method: { type: string, required: true }
    response:
      type: object
      properties:
        status_code: { type: integer, required: true }
        body: { type: object, required: false }

# Output schema - defines what the module returns
output:
  type: object
  required: true
  properties:
    status_code: { type: integer, required: true }
    body: { type: object, required: false }
    headers: { type: object, required: false }

# Module logic - actual behavior implementation
steps:
  - assert: !phs setup.path == main.path && setup.method == main.method
    then:
      payload: !phs setup.default_response
```

**Variables available in Phlow modules:**
- **`setup`**: Contains the configuration from the `with` section
- **`main`**: Contains the runtime input data
- **`payload`**: Data passed between steps

**Usage example:**

```phlow
modules:
  - module: ./route         # References route.phlow
    name: route_get_users   # Instance name
    with:                   # Configuration (becomes 'setup')
      path: /users
      method: GET
      default_response:
        status_code: 200
        body: []

steps:
  - use: route_get_users    # Uses the configured module instance
    input: !phs main        # Runtime data (becomes 'main')
```

### Benefits

#### Compiled Rust Modules
- **Maximum Performance**: Native speed and memory efficiency
- **System Integration**: Full access to system APIs and libraries
- **Type Safety**: Rust's compile-time guarantees

#### Phlow Modules
- **Rapid Development**: No compilation step required
- **Simplicity**: Pure Phlow syntax, no Rust knowledge needed
- **Portability**: Works across all platforms without recompilation
- **Live Editing**: Changes take effect immediately
- **Debugging**: Easy to debug with standard Phlow tools
- **Schema Validation**: Automatic input/output validation
- **Composability**: Can use all Phlow features (PHS, includes, etc.)

#### General Benefits (Both Types)
- **No Download**: Local modules skip the download process, improving startup time
- **Development Workflow**: Test changes immediately without publishing
- **Version Control**: Keep custom modules alongside your project
- **Offline Support**: Work without internet connectivity

## Good practice

A good practice is to keep the modules in a separate file, such as `modules.phlow`, and reference them in the `main.phlow` using `!include modules.phlow`. This helps maintain organized and easily maintainable code.

#### Example of `modules.phlow`:

```phlow
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

#### Example of `main.phlow` referencing `modules.phlow`:

```phlow
modules: !include modules.phlow
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
