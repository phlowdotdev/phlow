---
sidebar_position: 5
title: Special Directives
---
Phlow introduces special YAML directives that enable advanced functionality for data manipulation and scripting. Below are the available directives, explained with detailed examples.

### `!include`

The `!include` directive allows you to include the content of another YAML file. This is useful for organizing configurations or data into separate files and reusing them.

#### Basic Example:

```yaml
modules: !include modules.yaml
```

#### Example with Multiple Includes:

```yaml
config:
  database: !include configs/database.yaml
  server: !include configs/server.yaml
```

#### Structure of the Included File (`modules.yaml`):

```yaml
- module1
- module2
- module3
```

Resulting YAML:

```yaml
modules:
  - module1
  - module2
  - module3
```

---
### `!phs`

The `!phs` directive allows you to execute inline scripts directly within YAML. It is used to capture and manipulate variables, perform calculations, execute assertions dynamically, and even call functions from modules.

#### Basic Example:

```yaml
assert: !phs main.force
return: !phs `${main.name} is a student`
```

#### Example with Calculations:

```yaml
calculated_value: !phs `${main.value} * 2`
message: !phs `The result is ${main.result}`
```

#### Example with Conditions:

```yaml
return: !phs `${main.score > 50 ? 'Pass' : 'Fail'}`
```

#### Example Calling Module Functions:

```yaml
payload: !phs `query("Select * from users where id = ${main.user_id}")`
```

---

### `!import`

The `!import` directive allows you to import a script file (`.phs`) for evaluation. This is useful for reusing complex logic across different parts of the project.

#### Basic Example:

```yaml
assert: !import scripts/condition.phs
```

#### Example with Multiple Imports:

```yaml
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

Resulting YAML:

```yaml
assert: true
```

---

## Conclusion
These directives provide powerful tools for managing and manipulating data within Phlow. By using `!include`, `!phs`, and `!import`, you can create modular, reusable, and maintainable workflows that enhance the overall functionality of your projects.

> ### Additional Notes
> - Ensure that the paths provided in `!include` and `!import` are correct relative to the file where they are used.
> - The `!phs` directive can be used for both inline calculations and calling external scripts, making it versatile for various use cases.
> - When using `!import`, ensure that the script file is valid and contains the expected logic to avoid runtime errors.
> - The `!include` directive can be used to include YAML files, while `!import` is specifically for `.phs` script files.
