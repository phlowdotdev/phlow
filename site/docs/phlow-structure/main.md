---
sidebar_position: 2
title: Main Flow Structure (main.yaml)
---
The `main.yaml` file defines the flow's metadata and execution logic. By default, `main.yaml` is the primary file executed in Phlow, serving as the entry point for defining workflows and their associated configurations.

## Structure

| Field        | Description |
|--------------|-------------|
| `main`       | Specifies the main module, typically providing initial context (e.g., `cli` for command-line arguments). |
| `name`       | A user-friendly name for the flow. |
| `version`    | The flow's semantic version. |
| `description`| A brief summary of the flow's purpose. |
| `modules`    | A list of required modules (loaded using `!include`). |
| `steps`      | The sequence of steps the flow will execute. |


### Example

```yaml
main: cli
name: Are you a student?
version: 1.0.0
description: Check if you are a student.
author: Your Name
modules: !include modules.yaml
steps:
  # Flow execution steps
```

## Custom Fields

It is possible to add custom fields to the `main.yaml` file to better suit your workflow's needs. For example, you can include fields like `tags` to categorize your flow or `author` to specify the creator of the flow.

These fields can help improve the organization and documentation of your workflows.

Example with custom fields:

```yaml
main: cli
name: Are you a student?
version: 1.0.0
description: Check if you are a student.
author: Your Name
tags:
    - education
    - student
modules: !include modules.yaml
steps:
    # Flow execution steps
```
