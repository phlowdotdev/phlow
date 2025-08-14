---
sidebar_position: 5
title: Special Directives
---
Phlow introduces special directives that enable advanced functionality for data manipulation and scripting. Below are the available directives, explained with detailed examples.

### `!include`

The `!include` directive allows you to include the content of another Phlow file. This is useful for organizing configurations or data into separate files and reusing them. Additionally, you can pass arguments to the included files for dynamic content generation.

#### Basic Example:

```phlow
modules: !include modules.phlow
```

#### Example with Arguments:

```phlow
steps:
  !include ./handlers/auth.phlow target=user_login output='!phs user_data'
```

#### Example with Multiple Arguments:

```phlow
steps:
  !include ./templates/response.phlow status=200 message='Success' data='!phs payload'
```

#### Structure of the Included File with Arguments (`auth.phlow`):

```phlow
- use: !arg target
  input: !phs main
- log: "Processing authentication for: !arg target"
- assert: !phs !is_empty(credentials)
  then:
    return: !arg output
  else:
    return:
      error: "Authentication failed"
```

#### Argument Syntax:

- **Simple values**: `key=value`
- **Quoted values**: `key='value with spaces'` or `key="quoted value"`
- **PHS expressions**: `key='!phs expression'`

When using `!arg` in the included file, it will be replaced by the corresponding argument value passed in the `!include` directive **during compile time**. This means the substitution happens before the flow starts executing, creating a fully resolved Phlow structure.

#### Example with Multiple Includes:

```phlow
config:
  database: !include configs/database.phlow
  server: !include configs/server.phlow
```

#### Structure of the Included File (`modules.phlow`):

```phlow
- module1
- module2
- module3
```

Resulting structure:

```phlow
modules:
  - module1
  - module2
  - module3
```

---
### `!phs`

The `!phs` directive allows you to execute inline scripts directly within Phlow files. It is used to capture and manipulate variables, perform calculations, execute assertions dynamically, and even call functions from modules.

#### Basic Example:

```phlow
assert: !phs main.force
return: !phs `${main.name} is a student`
```

#### Example with Calculations:

```phlow
calculated_value: !phs `${main.value} * 2`
message: !phs `The result is ${main.result}`
```

#### Example with Conditions:

```phlow
return: !phs `${main.score > 50 ? 'Pass' : 'Fail'}`
```

#### Code Blocks

You can write multi-line code blocks using curly braces `{}`. These blocks are automatically unified into a single line during processing:

```phlow
steps:
  - payload: !phs {
      let user = main.user;
      let score = user.score || 0;
      let bonus = score > 80 ? 10 : 0;
      
      score + bonus
    }
    
  - use: log
    input:
      message: !phs {
        let result = payload * 2;
        let status = result > 100 ? "high" : "normal";
        
        `User score: ${result} (${status})`
      }
```

**Resulting transformation:**
```phlow
steps:
  - payload: "{{ { let user = main.user; let score = user.score || 0; let bonus = score > 80 ? 10 : 0; score + bonus } }}"
    
  - use: log
    input:
      message: "{{ { let result = payload * 2; let status = result > 100 ? \"high\" : \"normal\"; `User score: ${result} (${status})` } }}"
```

#### Advanced Code Block Examples:

**Data Processing:**
```phlow
- payload: !phs {
    let users = main.users || [];
    let activeUsers = users.filter(u => u.active);
    let summary = {
      total: users.length,
      active: activeUsers.length,
      inactive: users.length - activeUsers.length
    };
    
    summary
  }
```

**Complex Calculations:**
```phlow
- payload: !phs {
    let price = main.price;
    let discount = main.discount || 0;
    let tax = 0.18;
    
    let discountedPrice = price * (1 - discount);
    let finalPrice = discountedPrice * (1 + tax);
    
    {
      original: price,
      discount: discount,
      tax: tax,
      final: Math.round(finalPrice * 100) / 100
    }
  }
```

#### Code Block Limitations:

- **No function declarations**: You cannot declare functions inside code blocks
- **No imports**: External modules must be imported using `!import`
- **Single expression result**: The block should result in a single value

#### When to Use Code Blocks vs `!import`:

**Use Code Blocks for:**
- Short, inline calculations
- Data transformations
- Conditional logic
- Simple variable manipulations

**Use `!import` for:**
- Complex function definitions
- Reusable logic across multiple files
- Advanced algorithms
- Functions that need to be shared

#### Example Calling Module Functions:

```phlow
payload: !phs `query("Select * from users where id = ${main.user_id}")`
```

---

### `!arg`

The `!arg` directive is used within files that are included via `!include` to access arguments passed from the parent file. This directive is processed at **compile time** during Phlow transformation, not as a runtime variable. This means the argument values are resolved and substituted before the flow execution begins.

#### Basic Example:

**Parent file (`main.phlow`):**
```phlow
steps:
  !include ./handler.phlow target=user_authentication method=POST
```

**Included file (`handler.phlow`):**
```phlow
- use: !arg target
  input: !phs main
- log: "Processing !arg target with method !arg method"
- return: 
    target: !arg target
    method: !arg method
    processed: true
```

**Resulting structure after compilation:**
```phlow
- use: user_authentication
  input: !phs main
- log: "Processing user_authentication with method POST"
- return: 
    target: user_authentication
    method: POST
    processed: true
```

#### Error Handling:

If a required argument is not provided, Phlow will display a clear error message:

```
❌ Phlow Transformation Errors:
  1. Error including file ./handler.phlow: Missing required argument: 'output'

❌ Failed to transform Phlow file: /path/to/main.phlow
```

#### Advanced Example with Conditional Logic:

**Parent file:**
```phlow
steps:
  !include ./response.phlow status=200 success=true message='Operation completed'
```

**Included file (`response.phlow`):**
```phlow
- payload:
    status_code: !arg status
    success: !arg success
    message: !arg message
    timestamp: !phs now()
```

**Resulting structure after compilation:**
```phlow
- payload:
    status_code: 200
    success: true
    message: 'Operation completed'
    timestamp: !phs now()
```

#### Important Notes:

- **Compile-time processing**: All `!arg` references are resolved during Phlow transformation, before runtime execution
- **Static substitution**: Arguments are directly substituted into the Phlow structure
- **No runtime overhead**: Since processing happens at compile time, there's no performance impact during flow execution
- **Template-like behavior**: This enables creating reusable Phlow templates that are customized with specific values

---

### `!import`

The `!import` directive allows you to import a script file (`.phs`) for evaluation. This is useful for reusing complex logic across different parts of the project.

#### Basic Example:

```phlow
assert: !import scripts/condition.phs
```

#### Example with Multiple Imports:

```phlow
scripts:
  validate: !import scripts/validate.phs
  process: !import scripts/process.phs
```

#### Structure of the Imported File (`scripts/condition.phs`):

```phs
if (main.value > 10) {
  return true;
} else {
  return false;
}
```

Resulting structure:

```phlow
assert: true
```

---

## Conclusion
These directives provide powerful tools for managing and manipulating data within Phlow. By using `!include` with arguments, `!arg`, `!phs`, and `!import`, you can create modular, reusable, and maintainable workflows that enhance the overall functionality of your projects.

The argument system for `!include` enables you to create reusable templates and handlers that can be customized with different parameters, making your workflows more flexible and maintainable.

> ### Additional Notes
> - Ensure that the paths provided in `!include` and `!import` are correct relative to the file where they are used.
> - When using `!include` with arguments, all required `!arg` references in the included file must have corresponding arguments provided.
> - **`!arg` is processed at compile time**: All argument substitutions happen during Phlow transformation, not during runtime execution.
> - The `!phs` directive can be used for both inline calculations and calling external scripts, making it versatile for various use cases.
> - When using `!import`, ensure that the script file is valid and contains the expected logic to avoid runtime errors.
> - The `!include` directive can be used to include Phlow files, while `!import` is specifically for `.phs` script files.
> - Arguments in `!include` support simple values, quoted strings, and PHS expressions for maximum flexibility.
