---
sidebar_position: 2
title: CLI Module
hide_title: true
---

# CLI Module

The CLI module provides a complete command-line interface for Phlow applications, allowing declarative definition, parsing and validation of arguments through Phlow configuration.

## 🚀 Features

### Key Features

- ✅ **Declarative definition** of arguments via Phlow
- ✅ **Automatic parsing** of positional and optional arguments
- ✅ **Type validation** (string, integer, boolean)
- ✅ **Support for flags** long (--flag) and short (-f)
- ✅ **Configurable default values**
- ✅ **Required and optional arguments**
- ✅ **Automatic help** with colored formatting
- ✅ **Error handling** with clear messages
- ✅ **Complete observability** with OpenTelemetry
- ✅ **Unknown flag validation**
- ✅ **Support for positional arguments** by index

## 📋 Configuration

### Basic Configuration

```phlow
name: "my-cli-app"
version: "1.0.0"
description: "A sample CLI application"
author: "Name <email@example.com>"
license: "MIT"
main: "cli_handler"

modules:
  - name: "cli_handler"
    module: "cli"
    with:
      additional_args: false
      args:
        - name: "input_file"
          long: "input"
          short: "i"
          help: "Input file"
          type: "string"
          required: true
          
        - name: "output_file"
          long: "output"
          short: "o"
          help: "Output file"
          type: "string"
          required: false
          default: "output.txt"
          
        - name: "verbose"
          long: "verbose"
          short: "v"
          help: "Verbose mode"
          type: "boolean"
          required: false
          default: false
          
        - name: "count"
          long: "count"
          short: "c"
          help: "Number of iterations"
          type: "integer"
          required: false
          default: 1
```

### Configuration with Positional Arguments

```phlow
modules:
  - name: "cli_handler"
    module: "cli"
    with:
      args:
        - name: "command"
          index: 0
          help: "Command to execute"
          type: "string"
          required: true
          
        - name: "target"
          index: 1
          help: "Command target"
          type: "string"
          required: false
          default: "default"
          
        - name: "debug"
          long: "debug"
          short: "d"
          help: "Enable debug"
          type: "boolean"
          required: false
```

## 🔧 Configuration Parameters

### General Configuration
- `additional_args` (boolean, optional): If enabled, does not validate unmapped arguments (default: false)
- `args` (array, required): List of arguments to be processed

### Argument Configuration

Each argument can have the following properties:

- `name` (string, required): Internal name of the argument
- `long` (string, optional): Long flag name (--example)
- `short` (string, optional): Short flag name (-e)
- `help` (string, optional): Help text
- `type` (enum, required): Argument type [string, integer, boolean]
- `required` (boolean, optional): If the argument is required (default: false)
- `default` (any, optional): Default value
- `index` (integer, optional): Index for positional arguments

## 💻 Usage Examples

### Command with Flags

```bash
# Using long flags
./myapp --input file.txt --output result.txt --verbose --count 5

# Using short flags
./myapp -i file.txt -o result.txt -v -c 5

# Mixing flags
./myapp --input file.txt -o result.txt -v
```

### Command with Positional Arguments

```bash
# Positional arguments
./myapp create user --debug

# Equivalent to:
# command = "create"
# target = "user"  
# debug = true
```

### Automatic Help

```bash
./myapp --help
./myapp -h
./myapp -H
```

## 🎨 Help Output

The module automatically generates formatted and colored help output:

```
Usage: My CLI App
       Version: 1.0.0
       Description: A sample CLI application
       License: MIT
       Author: Name <email@example.com>
       Homepage: https://example.com
       Repository: https://github.com/user/repo

Arguments:
  command [required]  Command to execute
  target [optional] default  Command target

Options:
  --input, -i input_file [required]  Input file
  --output, -o output_file [optional] output.txt  Output file
  --verbose, -v verbose [optional] false  Verbose mode
  --count, -c count [optional] 1  Number of iterations
  --debug, -d debug [optional]  Enable debug
```

## 📊 Observability

The module automatically generates OpenTelemetry spans with the following attributes:

### Span Attributes
- `otel.name`: Application name
- `process.executable.name`: Executable name
- `process.exit.code`: Exit code
- `error.type`: Error type (if any)
- `process.pid`: Process ID
- `process.command_args`: Command arguments
- `process.executable.path`: Executable path

## 🔍 Data Output

The module processes arguments and returns an object with all values:

```json
{
  "input_file": "data.txt",
  "output_file": "result.txt",
  "verbose": true,
  "count": 5,
  "command": "create",
  "target": "user",
  "debug": false
}
```

## 🛠️ Error Handling

The module provides comprehensive error handling:

### Missing Required Arguments
```
Error:
       Missing required argument: input_file
```

### Unknown Flags
```
Error:
       Unknown flag: --unknown. Use --help to see the available flags.
```

### Invalid Values
```
Error:
       Invalid value for count: not_a_number
```

### Invalid Positional Arguments
```
Error:
       Invalid value for positional argument command: cannot start with '-' or '--'. Found '--invalid'
```

## 🌐 Complete Example

```phlow
name: "file-processor"
version: "2.1.0"
description: "File processor with multiple options"
author: "Dev Team <dev@company.com>"
license: "MIT"
homepage: "https://company.com/file-processor"
repository: "https://github.com/company/file-processor"
main: "cli_processor"

modules:
  - name: "cli_processor"
    module: "cli"
    with:
      additional_args: false
      args:
        # Positional arguments
        - name: "action"
          index: 0
          help: "Action to execute [process, validate, convert]"
          type: "string"
          required: true
          
        - name: "file_path"
          index: 1
          help: "Path to file to process"
          type: "string"
          required: true
          
        # Optional flags
        - name: "output_dir"
          long: "output"
          short: "o"
          help: "Output directory"
          type: "string"
          required: false
          default: "./output"
          
        - name: "format"
          long: "format"
          short: "f"
          help: "Output format [json, xml, csv]"
          type: "string"
          required: false
          default: "json"
          
        - name: "batch_size"
          long: "batch-size"
          short: "b"
          help: "Batch size for processing"
          type: "integer"
          required: false
          default: 100
          
        - name: "verbose"
          long: "verbose"
          short: "v"
          help: "Verbose mode"
          type: "boolean"
          required: false
          
        - name: "dry_run"
          long: "dry-run"
          help: "Execute without making changes"
          type: "boolean"
          required: false

steps:
  - name: "validate_action"
    condition:
      left: "args.action"
      operator: "in"
      right: ["process", "validate", "convert"]
    else:
      return: "Invalid action. Use: process, validate or convert"
      
  - name: "process_file"
    # Processing logic based on arguments
    script: |
      // Access arguments via args.argument_name
      let action = args.action;
      let file_path = args.file_path;
      let output_dir = args.output_dir;
      let format = args.format;
      let batch_size = args.batch_size;
      let verbose = args.verbose || false;
      let dry_run = args.dry_run || false;
      
      // Processing logic
      `Processing ${file_path} with action ${action}`;
```

### Example Usage

```bash
# Basic processing
./file-processor process data.txt

# With options
./file-processor process data.txt --output ./results --format xml --batch-size 50 --verbose

# Using short flags
./file-processor convert input.csv -o ./converted -f json -b 25 -v

# Dry run
./file-processor validate config.yaml --dry-run --verbose

# Help
./file-processor --help
```

## 🔒 Validation

- **Data types**: Automatic validation of string, integer and boolean
- **Required arguments**: Presence verification
- **Valid flags**: Rejection of unknown flags
- **Default values**: Automatic application when not provided
- **Positional arguments**: Index and order validation

## 🏷️ Tags

- cli
- command-line
- arguments
- parsing
- validation

---

**Version**: 0.0.1  
**Author**: Philippe Assis `<codephilippe@gmail.com>`
**License**: MIT  
**Repository**: https://github.com/phlowdotdev/phlow

