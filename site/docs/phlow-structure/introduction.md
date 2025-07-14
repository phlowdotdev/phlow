---
sidebar_position: 1
title: Introduction
hide_title: true
---
# Phlow Structure

Phlow is a modular runtime for building backends, automations, and orchestrations using YAML to define **flows**. Each flow is descriptive and structured into **modules**, **steps**, and **scripts**.

This document covers:

- Main structure (`main.phlow`)
- Modules inclusion (`modules.yaml`)
- Dynamic scripts (`.phs`)
- Special commands (`!include`, `!phs`, `!import`)

### Example: Student Flow

To better understand the concepts, refer to the [Student Flow example](https://github.com/phlowdotdev/phlow/examples/students). This example demonstrates how to implement a flow that evaluates whether someone is a student based on their age and other parameters.

```yaml
main: cli
name: Are you a student?
version: 1.0.0
description: Check if you are a student.
author: Your Name
modules: !include modules.yaml
steps:
  - assert: !phs main.force
    then:
      return: !phs `${main.name} is a student, but the age is not valid`
    else:
      - assert: !phs main.age < 18 && main.age > 3
        then:
          return: !phs `${main.name} is a student`
      - assert: !import scripts/condition.phs
        then:
          return: !phs `${main.name} is not a student`
        then:
          return: !phs `${main.name} is not a student`
      - assert: !phs main.age <= 3
        then:
          return: !phs `${main.name} is a baby`  
      - return: !import scripts/output.phs
```

###  Flow Logic Overview

The "Are you a student?" flow operates as follows:

1. Captures CLI arguments: `name`, `age`, `force`.
2. If `force` is `true`, returns a message indicating the user is a student.
3. Otherwise:
    - If `age` is 18 or older, returns a message indicating the user is a student.
    - If `age` is less than 3, indicates the person is a baby.
    - Otherwise, returns an invalid age message.


###  Structure Visualization

```
main.phlow
 ├── Header (main, name, version, etc.)
 ├── modules: !include modules.yaml (optional)
 └── steps: conditional execution flow

modules.yaml (optional)
 └── CLI arguments definition

scripts/ (optional)
 ├── condition.phs (age validation)
 └── output.phs (error message)
```

## Simplified Flows

Phlow also supports simplified flows without a main module or external modules, perfect for simple automation:

```yaml
name: Simple Processing
version: 1.0.0
description: Process data without external dependencies
steps:
  - payload:
      data: "Processing this information"
  - payload: !phs `Result: ${ payload.data } - completed`
```
