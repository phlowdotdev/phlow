---
sidebar_position: 1
title: Advanced Syntax
---

# Advanced Phlow Syntax

:::warning Alpha Version
Phlow is currently in **alpha** stage and under active development. It is **not recommended for production use** at this time. The stable version will be released as **v1.0.0 or higher** in the future.
:::

This document provides a comprehensive guide to Phlow's advanced syntax features, including the preprocessor transformations, scripting capabilities, and special directives.

## Overview

Phlow uses a **4-stage preprocessor pipeline** that transforms your `.phlow` files before execution:

1. **Directives Processing** - Handles `!include`, `!import`, `!arg`
2. **Auto PHS Detection** - Automatically adds `!phs` where needed
3. **Script Evaluation** - Converts `!phs` blocks to executable format
4. **Module Transformation** - Converts shorthand module calls to standard format

## Preprocessor Pipeline

### Stage 1: Directives Processing

#### `!include` Directive
Include content from other `.phlow` files with optional arguments.

**Basic Include:**
```yaml
modules: !include modules.phlow
```

**Include with Arguments:**
```yaml
# main.phlow
steps: !include template.phlow name=john age=25

# template.phlow
- return: !phs `Hello !arg name, you are !arg age years old`
```

**Block Include (with indentation):**
```yaml
steps:
  !include common-steps.phlow
```

#### `!import` Directive
Import script files as inline strings.

```yaml
steps:
  - assert: !import validation.phs
  - condition: !import scripts/check-user.rhai
```

**File Extensions:**
- `.phs` or `.rhai` → Imported as `"{{ script_content }}"`
- Other extensions → Imported as `"script_content"`

#### `!arg` Directive
Use arguments passed from `!include` calls.

```yaml
# template.phlow
name: User Template
steps:
  - log:
      message: !phs `Processing user !arg username with role !arg role`
```

### Stage 2: Auto PHS Detection

The preprocessor automatically detects when code needs `!phs` prefixes and adds them.

#### Values That Get Auto-PHS

**Mathematical Operations:**
```yaml
# Before preprocessing
- payload: 10 + 20
- assert: main.age > 18

# After preprocessing  
- payload: !phs 10 + 20
- assert: !phs main.age > 18
```

**Variable References:**
```yaml
# Before
- return: main.name
- payload: payload.data

# After
- return: !phs main.name  
- payload: !phs payload.data
```

**Template Strings (any backticks):**
```yaml
# Before
- message: `OK`
- return: `Hello ${main.name}`

# After
- message: !phs `OK`
- return: !phs `Hello ${main.name}`
```

**Comparison Operations:**
```yaml
# Before
- assert: main.score >= 100
- condition: payload.status == "active"

# After
- assert: !phs main.score >= 100
- condition: !phs payload.status == "active"
```

**Logical Operations:**
```yaml
# Before
- assert: main.age > 18 && main.verified
- condition: status == "ok" || retry_count < 3

# After
- assert: !phs main.age > 18 && main.verified
- condition: !phs status == "ok" || retry_count < 3
```

**Code Blocks:**
```yaml
# Before
- payload: {
    let x = main.value * 2;
    x + 10
  }

# After
- payload: !phs {
    let x = main.value * 2; 
    x + 10
  }
```

#### Values That DON'T Get Auto-PHS

**Quoted Strings:**
```yaml
- message: "Hello World"    # No !phs needed
- name: 'John Doe'          # No !phs needed
```

**Numbers:**
```yaml
- count: 42                 # No !phs needed
- price: 19.99             # No !phs needed
- negative: -100           # No !phs needed
```

**Booleans:**
```yaml
- enabled: true            # No !phs needed
- active: false           # No !phs needed
```

**Null Values:**
```yaml
- data: null              # No !phs needed
- optional: ~             # No !phs needed (YAML null)
```

**Existing Directives:**
```yaml
- steps: !include other.phlow  # Already has directive
- script: !import check.phs    # Already has directive
```

#### Affected Properties

Auto-PHS is applied to specific properties:
- `assert:`
- `return:`
- `payload:`
- `input:`
- `message:`

### Stage 3: Script Evaluation

Converts `!phs` blocks into executable format for the Rhai engine.

#### Inline Scripts
```yaml
# Input
- assert: !phs main.age > 18

# Output
- assert: "{{ main.age > 18 }}"
```

#### Code Blocks with Braces
```yaml
# Input
- payload: !phs {
    let user = main.user;
    let score = user.points * 2;
    { name: user.name, total: score }
  }

# Output  
- payload: "{{ { let user = main.user; let score = user.points * 2; { name: user.name, total: score } } }}"
```

#### Markdown-Style Code Blocks
````yaml
# Input
- script: !phs ```
    let data = fetch_user(main.id);
    process_data(data)
  ```

# Output
- script: "{{ let data = fetch_user(main.id); process_data(data) }}"
````

#### Indented Code Blocks
```yaml
# Input
- transform: !phs
    let items = payload.items;
    items.map(|item| item * 2)

# Output
- transform: "{{ let items = payload.items; items.map(|item| item * 2) }}"
```

### Stage 4: Module Transformation

Converts shorthand module syntax to standard `use` + `input` format.

#### Module Shorthand
```yaml
# Before transformation
modules:
  - module: http_request
  - module: log

steps:
  - http_request:
      url: "https://api.example.com"
      method: GET
  - log:
      message: "Request completed"

# After transformation  
steps:
  - use: http_request
    input:
      url: "https://api.example.com"
      method: GET
  - use: log
    input:
      message: "Request completed"
```

#### Properties NOT Transformed

These are exclusive system properties and won't be converted:
- `use`, `to`, `id`, `label`
- `assert`, `condition` 
- `return`, `payload`, `input`
- `then`, `else`, `steps`

## Scripting with Rhai

Phlow uses the [Rhai scripting language](https://rhai.rs) for dynamic expressions.

### Variable Context

Available variables in scripts:
- `main` - CLI arguments or main context
- `payload` - Current step payload
- Previous step results (by index or label)

### Data Types

**Basic Types:**
```yaml
- payload: !phs 42                    # Integer
- payload: !phs 3.14159              # Float  
- payload: !phs "Hello"              # String
- payload: !phs true                 # Boolean
- payload: !phs [1, 2, 3]           # Array
```

**Objects:**
```yaml
- payload: !phs {
    name: "John",
    age: 30,
    skills: ["rust", "javascript"]
  }
```

**Template Strings:**
```yaml
- message: !phs `User ${main.name} has ${payload.points} points`
```

### Advanced Scripting Examples

#### Conditional Logic
```yaml
- payload: !phs {
    let age = main.age;
    if age >= 18 {
      { status: "adult", eligible: true }
    } else {
      { status: "minor", eligible: false }  
    }
  }
```

#### Array Operations
```yaml
- payload: !phs {
    let items = payload.data;
    let filtered = items.filter(|item| item.active);
    let mapped = filtered.map(|item| {
      name: item.name,
      score: item.points * 2
    });
    mapped
  }
```

#### Complex Data Transformation
```yaml
- payload: !phs {
    let user_data = main.users;
    let processed = user_data.map(|user| {
      let bonus = if user.premium { 100 } else { 50 };
      {
        id: user.id,
        name: user.name.to_upper(),
        total_score: user.score + bonus,
        level: if user.score > 1000 { "expert" } else { "beginner" }
      }
    });
    
    {
      users: processed,
      count: processed.len(),
      timestamp: timestamp()
    }
  }
```

## Best Practices

### 1. Use Auto-PHS When Possible
Let the preprocessor handle `!phs` insertion automatically:

```yaml
# ✅ Good - Auto-PHS will handle this
- assert: main.age > 18
- return: `Welcome ${main.name}`

# ❌ Unnecessary - Manual !phs not needed
- assert: !phs main.age > 18
- return: !phs `Welcome ${main.name}`
```

### 2. Organize Complex Scripts
Break complex logic into separate files:

```yaml
# main.phlow
steps:
  - assert: !import scripts/validate-user.phs
  - payload: !import scripts/transform-data.phs
```

### 3. Use Meaningful Variable Names
```yaml
# ✅ Good
- payload: !phs {
    let user_age = main.age;
    let is_adult = user_age >= 18;
    { eligible: is_adult, category: if is_adult { "adult" } else { "minor" } }
  }

# ❌ Hard to read
- payload: !phs {
    let x = main.age;
    let y = x >= 18;
    { a: y, b: if y { "adult" } else { "minor" } }
  }
```

### 4. Leverage Template Includes
Create reusable templates:

```yaml
# templates/user-validation.phlow
- assert: !arg field_name
  then:
    - return: !phs `${!arg field_name} is valid`
  else:
    - return: !phs `${!arg field_name} is invalid`

# main.phlow
steps:
  !include templates/user-validation.phlow field_name=main.email
```

## Debugging

### Print Transformed YAML
Use `--print` to see preprocessor output. Choose the format with `--output yaml|json`:

```bash
phlow main.phlow --print --output yaml
phlow main.phlow --print --output json
```

### Common Issues

**1. Missing Auto-PHS Detection:**
```yaml
# If auto-PHS doesn't detect, add manually:
- custom_property: !phs main.value + 10
```

**2. Template String Escaping:**
```yaml
# For strings with quotes, use backticks:
- message: !phs `He said "Hello World"`
```

**3. Complex Object Construction:**
```yaml
# Use proper Rhai object syntax:
- payload: !phs {
    user: { 
      name: main.name, 
      data: [1, 2, 3] 
    },
    meta: { timestamp: timestamp() }
  }
```

This advanced syntax guide covers the complete preprocessor pipeline and scripting capabilities. The automatic transformations make Phlow flows more readable while maintaining full scripting power when needed.
