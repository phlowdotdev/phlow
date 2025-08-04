---
sidebar_position: 3
title: CLI Applications
---

# CLI Applications

Phlow makes it easy to create command-line interfaces (CLI) applications with argument parsing, validation, and interactive features.

## Basic CLI Application

Create a simple CLI tool that greets users:

```phlow title="greet-cli.phlow"
name: Greeting CLI
version: 1.0.0
description: A simple greeting CLI application

main: cli
modules:
  - module: cli
    version: latest
    with:
      args:
        - index: 1
          long: name
          type: string
          required: true
          help: Name of the person to greet
        - index: 2
          long: greeting
          type: string
          default: "Hello"
          help: Type of greeting
        - long: uppercase
          short: u
          type: boolean
          default: false
          help: Make greeting uppercase

steps:
  - payload: !phs `${main.greeting}, ${main.name}!`
  
  - assert: !phs main.uppercase
    then:
      payload: !phs payload.toUpperCase()
  
  - return: !phs payload
```

### Usage Examples

```bash
# Basic usage
phlow greet-cli.phlow John
# Output: Hello, John!

# Custom greeting
phlow greet-cli.phlow John "Good morning"
# Output: Good morning, John!

# With uppercase flag
phlow greet-cli.phlow John "Hi there" --uppercase
# Output: HI THERE, JOHN!

# Short flag
phlow greet-cli.phlow John "Hey" -u
# Output: HEY, JOHN!
```

## File Processing CLI

Create a CLI tool that processes files:

```phlow title="file-processor.phlow"
name: File Processor
version: 1.0.0
description: Process text files with various operations

main: cli
modules:
  - module: cli
    version: latest
    with:
      args:
        - index: 1
          long: input_file
          type: string
          required: true
          help: Input file path
        - index: 2
          long: output_file
          type: string
          required: false
          help: Output file path
        - long: operation
          short: o
          type: string
          default: "count"
          help: Operation to perform
        - long: verbose
          short: v
          type: boolean
          default: false
          help: Enable verbose output

steps:
  - assert: !phs main.verbose
    then:
      - log:
          message: !phs `Processing file: ${main.input_file}`
      - log:
          message: !phs `Operation: ${main.operation}`
  
  # Read file (simulated - in real use, you'd use a file module)
  - payload: !phs {
      content: "This is sample file content\nWith multiple lines\nFor processing",
      operation: main.operation,
      input_file: main.input_file,
      output_file: main.output_file
    }
  
  # Process based on operation
  - assert: !phs payload.operation == "count"
    then:
      - payload: !phs {
          ...payload,
          result: {
            lines: payload.content.split('\n').length,
            words: payload.content.split(/\s+/).length,
            characters: payload.content.length
          }
        }
  
  - assert: !phs payload.operation == "uppercase"
    then:
      - payload: !phs {
          ...payload,
          result: payload.content.toUpperCase()
        }
  
  - assert: !phs payload.operation == "lowercase"
    then:
      - payload: !phs {
          ...payload,
          result: payload.content.toLowerCase()
        }
  
  - assert: !phs payload.operation == "reverse"
    then:
      - payload: !phs {
          ...payload,
          result: payload.content.split('').reverse().join('')
        }
  
  - assert: !phs main.verbose
    then:
      log:
        message: !phs `Operation completed: ${payload.operation}`
  
  - return: !phs payload.result
```

## Interactive CLI with Validation

Create an interactive CLI with input validation:

```phlow title="user-registration.phlow"
name: User Registration CLI
version: 1.0.0
description: Interactive user registration with validation

main: cli
modules:
  - module: cli
    version: latest
    with:
      args:
        - index: 1
          long: email
          type: string
          required: true
          help: User email address
        - index: 2
          long: age
          type: number
          required: true
          help: User age
        - long: role
          short: r
          type: string
          default: "user"
          help: User role
        - long: active
          type: boolean
          default: false
          help: Activate user immediately
        - long: config
          short: c
          type: string
          required: false
          help: Configuration file path

steps:
  # Validate email
  - assert: !phs main.email.includes('@') && main.email.includes('.')
    then:
      payload: !phs { email: main.email, email_valid: true }
    else:
      - log:
          message: !phs `Invalid email format: ${main.email}`
      - return: !phs { error: "Invalid email format" }
  
  # Validate age
  - assert: !phs main.age >= 18 && main.age <= 100
    then:
      payload: !phs { ...payload, age: main.age, age_valid: true }
    else:
      - log:
          message: !phs `Invalid age: ${main.age}. Must be between 18 and 100`
      - return: !phs { error: "Invalid age" }
  
  # Create user object
  - payload: !phs {
      user: {
        email: payload.email,
        age: payload.age,
        role: main.role,
        active: main.active,
        created_at: new Date().toISOString(),
        id: Math.random().toString(36).substr(2, 9)
      }
    }
  
  # Log configuration if provided
  - assert: !phs main.config
    then:
      log:
        message: !phs `Using configuration file: ${main.config}`
  
  - log:
      message: !phs `User registered successfully: ${payload.user.email}`
  
  - return: !phs payload.user
```

## CLI with Subcommands

Create a CLI with multiple subcommands:

```phlow title="project-manager.phlow"
name: Project Manager CLI
version: 1.0.0
description: Manage projects with subcommands

main: cli
modules:
  - module: cli
    version: latest
    with:
      args:
        - index: 1
          long: command
          type: string
          required: true
          help: Command to execute
        - index: 2
          long: project_name
          type: string
          required: false
          help: Project name
        - long: template
          short: t
          type: string
          default: "basic"
          help: Project template
        - long: force
          short: f
          type: boolean
          default: false
          help: Force operation

steps:
  # Initialize projects storage (simulated)
  - payload: !phs {
      projects: [
        { name: "my-app", template: "web", status: "active" },
        { name: "api-service", template: "api", status: "active" },
        { name: "old-project", template: "basic", status: "archived" }
      ]
    }
  
  # Handle create command
  - assert: !phs main.command == "create"
    then:
      - assert: !phs main.project_name
        then:
          - payload: !phs {
              ...payload,
              projects: [
                ...payload.projects,
                {
                  name: main.project_name,
                  template: main.template,
                  status: "active",
                  created_at: new Date().toISOString()
                }
              ]
            }
          - return: !phs {
              message: `Project '${main.project_name}' created successfully`,
              project: payload.projects[payload.projects.length - 1]
            }
        else:
          return: !phs { error: "Project name is required for create command" }
  
  # Handle list command
  - assert: !phs main.command == "list"
    then:
      return: !phs {
        projects: payload.projects,
        total: payload.projects.length
      }
  
  # Handle delete command
  - assert: !phs main.command == "delete"
    then:
      - assert: !phs main.project_name
        then:
          - payload: !phs {
              ...payload,
              projectExists: payload.projects.some(p => p.name === main.project_name)
            }
          - assert: !phs payload.projectExists
            then:
              - assert: !phs main.force || confirm("Are you sure you want to delete this project?")
                then:
                  - payload: !phs {
                      ...payload,
                      projects: payload.projects.filter(p => p.name !== main.project_name)
                    }
                  - return: !phs {
                      message: `Project '${main.project_name}' deleted successfully`
                    }
                else:
                  return: !phs { message: "Delete operation cancelled" }
            else:
              return: !phs { error: `Project '${main.project_name}' not found` }
        else:
          return: !phs { error: "Project name is required for delete command" }
  
  # Handle status command
  - assert: !phs main.command == "status"
    then:
      return: !phs {
        total_projects: payload.projects.length,
        active_projects: payload.projects.filter(p => p.status === "active").length,
        archived_projects: payload.projects.filter(p => p.status === "archived").length,
        templates: {
          basic: payload.projects.filter(p => p.template === "basic").length,
          web: payload.projects.filter(p => p.template === "web").length,
          api: payload.projects.filter(p => p.template === "api").length,
          cli: payload.projects.filter(p => p.template === "cli").length
        }
      }
```

### Usage Examples

```bash
# Create a new project
phlow project-manager.phlow create my-new-app --template web

# List all projects
phlow project-manager.phlow list

# Delete a project
phlow project-manager.phlow delete old-project --force

# Show status
phlow project-manager.phlow status
```

## Testing CLI Applications

Create tests for your CLI applications:

```phlow title="cli-test.phlow"
name: CLI Test Suite
version: 1.0.0
description: Testing CLI applications

tests:
  # Test basic greeting
  - main:
      name: "John"
      greeting: "Hello"
      uppercase: false
    payload: null
    assert: !phs payload == "Hello, John!"
  
  # Test uppercase flag
  - main:
      name: "Jane"
      greeting: "Hi"
      uppercase: true
    payload: null
    assert: !phs payload == "HI, JANE!"
  
  # Test user registration validation
  - main:
      email: "user@example.com"
      age: 25
      role: "user"
      active: true
    payload: null
    assert: !phs payload.user.email == "user@example.com"
  
  # Test invalid email
  - main:
      email: "invalid-email"
      age: 25
      role: "user"
      active: false
    payload: null
    assert: !phs payload.error == "Invalid email format"

steps:
  # Test greeting functionality
  - assert: !phs main.name && main.greeting
    then:
      - payload: !phs `${main.greeting}, ${main.name}!`
      - assert: !phs main.uppercase
        then:
          payload: !phs payload.toUpperCase()
      - return: !phs payload
  
  # Test user registration
  - assert: !phs main.email && main.age
    then:
      - assert: !phs main.email.includes('@') && main.email.includes('.')
        then:
          - assert: !phs main.age >= 18 && main.age <= 100
            then:
              return: !phs {
                user: {
                  email: main.email,
                  age: main.age,
                  role: main.role,
                  active: main.active,
                  created_at: new Date().toISOString(),
                  id: Math.random().toString(36).substr(2, 9)
                }
              }
            else:
              return: !phs { error: "Invalid age" }
        else:
          return: !phs { error: "Invalid email format" }
```

## Key Features Demonstrated

1. **Argument Parsing**: Define positional and optional arguments
2. **Type Validation**: Ensure arguments are of correct types
3. **Default Values**: Provide sensible defaults for optional arguments
4. **Choices**: Restrict arguments to specific values
5. **Boolean Flags**: Handle true/false options with short and long forms
6. **Subcommands**: Implement multiple operations in a single CLI
7. **Input Validation**: Validate user input and provide error messages
8. **Interactive Features**: Create user-friendly command-line interfaces
9. **Testing**: Automated testing of CLI functionality

These examples show how to build robust command-line applications with Phlow's CLI module.
