<p align="center"> <img src="../docs/phlow.svg" alt="Phlow logo" width="140"/> </p> <h1 align="center">PHS – Phlow Script</h1>


**PHS** is a lightweight scripting format for [Phlow](https://github.com/lowcarboncode/phlow), built on top of [Rhai](https://rhai.rs/). It enables simple, dynamic behavior scripting using `.phs` files while deeply integrating with the Phlow runtime and module system.

## ✨ Overview

PHS (Phlow Script) brings the power of embedded scripting to YAML-based workflows. It's designed to let you inject dynamic logic through readable scripts, while preserving Phlow's declarative style.

You can inject modules directly into your PHS context via the `modules` section of your `.yaml` configuration. Each module declared becomes globally accessible in the `.phs` script, making it easy to mix scripting with orchestrated steps.

## 🔌 Module Injection via YAML

All modules declared in the YAML under `modules:` are automatically available inside your `.phs` script. For example, when you load the `log` module, its functions can be used directly in the script.

## 🧪 Example

```yaml
# main.yaml
main: cli
name: Example Cli
version: 1.0.0
description: Example CLI module
author: Your Name

modules: 
  - module: cli
    version: latest
    with:
      additional_args: false
      args: 
        - name: name
          description: Name of the user
          index: 1
          type: string
          required: false
  - module: log
    version: latest

steps:
  - return: !import script.phs
```

```rust
log("warn", `Hello, ${main.name}`);
```

### 💡Output
If the user runs:
```bash
phlow run main.yaml --name Philippe
```

The script will log:
```bash
[warn] Hello, Philippe
```

## 📁 File Extensions
Phlow automatically loads `.phs` scripts when referenced in the flow via `!import`. These scripts are parsed and executed using the internal Rhai engine extended with Phlow modules.

### 🔐 Modules Supported in PHS
Any module that exposes scripting bindings can be used. Example modules:

- log
- cli
- http_server
- (and any custom Rust module registered with bindings)


## 🧠 Variables in PHS
You can declare and use variables in `.phs` scripts using the `let` keyword. These variables help you store temporary values, compose strings, perform calculations, or reuse values throughout your script.


### 🔤 Declaring Variables
```rust
let name = main.name;
let greeting = "Hello";
let message = `${greeting}, ${name}!`;

log("info", message);
```

### ✍️ Reassigning Values
Variables can be reassigned at any point:
```rust
let count = 1;
count = count + 1;
```

### 🔄 Using Function Results
You can assign the result of a function to a variable:
```rust
let status = "warn";
let msg = "Something happened";

log(status, msg);
```
## 🧱 A
PHS allows you to work with arrays and objects (maps) natively. These are useful when handling lists of items, grouping values, or building dynamic data structures.

### 📚 Arrays
You can create arrays using square brackets []:

```rust
let fruits = ["apple", "banana", "orange"];
log("info", `First fruit: ${fruits[0]}`);
➕ Adding Items

fruits.push("grape");
```

### 🔄 Looping Through Arrays
```rust
for fruit in fruits {
  log("debug", `Fruit: ${fruit}`);
}
```

### 🧳 Objects (Maps)
You can define key-value objects using curly braces {}:

```rust
let user = #{
  name: main.name,
  age: 30,
  active: true
};

log("info", `User: ${user.name} (age: ${user.age})`);
🔧 Updating Properties

user.age = 31;
user.status = "online";
```


### 📦 Nesting
Objects and arrays can be nested:

```rust
let config = #{
  tags: ["dev", "backend"],
  options: #{
    retries: 3,
    timeout: 1000
  }
};

log("debug", `Retries: ${config.options.retries}`);
```

## 🧭 Conditionals in PHS
PHS supports conditional logic using if, else if, and else blocks. These let you define dynamic behaviors based on data or user input.

### ✅ Basic If
```rust
if main.name == "Philippe" {
  log("info", "Welcome back, boss!");
}
```
### 🔁 If...Else
```rust
if main.name == "Alice" {
  log("info", "Hi Alice!");
} else {
  log("info", "Hello, guest!");
}
```
### 🔀 Else If
```rust
if main.name == "Bob" {
  log("info", "Hello Bob!");
} else if main.name == "Charlie" {
  log("info", "Hey Charlie!");
} else {
  log("info", "Who are you?");
}
```
### 🔗 Nested Conditions
```rust
if main.name != "" {
  if main.name.len > 5 {
    log("debug", "That's a long name.");
  } else {
    log("debug", "Short and sweet.");
  }
}
```

Conditionals are a great way to adapt the behavior of your script based on CLI arguments, environment values, or runtime results.



## 🔁 Loops in PHS
PHS supports looping structures to help you iterate over arrays or repeat actions multiple times. The most common loop you'll use is the for loop.

### 📚 Looping Through an Array
```rust
let fruits = ["apple", "banana", "orange"];

for fruit in fruits {
  log("info", `Fruit: ${fruit}`);
}
```
### 🔢 Looping with a Range
You can loop through a range of numbers:

```rust
for i in 0..5 {
  log("debug", `Index: ${i}`);
}
```
This prints numbers from 0 to 4.

### 🔄 Nested Loops
Loops can be nested for handling multi-dimensional data:

```rust
let matrix = [
  [1, 2],
  [3, 4]
];

for row in matrix {
  for value in row {
    log("debug", `Value: ${value}`);
  }
}
```

### 🛑 Breaking a Loop (not supported yet)
Currently, there's no support for break or continue in .phs. Keep your loops simple and controlled with conditions when needed.

Loops are powerful for automating repetitive tasks or handling collections of data. Combine them with conditionals and functions to build expressive scripts.

## 🧩 Functions in PHS
You can define your own functions in .phs to reuse logic, organize your code, and make scripts cleaner and more modular.

### 🛠 Defining a Function
Use the fn keyword:

```rust
fn greet(name) {
  log("info", `Hello, ${name}!`);
}
```
### ▶️ Calling a Function
Once defined, just call it like this:

```rust
greet("Philippe");
```
This will log:
```bash
[info] Hello, Philippe!
```
### ↩️ Returning Values
Functions can return values using return:
```rust
fn double(n) {
  return n * 2;
}

let result = double(5);
log("debug", `Result: ${result}`);
```

### 🧠 Functions with Logic
You can include conditionals, loops, and other functions inside your custom function:

```rust
fn log_fruits(fruits) {
  for fruit in fruits {
    log("info", `Fruit: ${fruit}`);
  }
}

let list = ["apple", "banana", "orange"];
log_fruits(list);
```

### ⚠️ Scope
Variables declared inside a function are local to that function unless returned or passed back explicitly.


# 🧬 PHS Syntax and Language Features

This guide expands on PHS (Phlow Script)'s syntax, types, and scripting features.

## 📐 Data Types in PHS

PHS supports common primitive types, plus arrays and maps (objects):

| Type     | Example               |
|----------|------------------------|
| `bool`   | `true`, `false`        |
| `string` | `"hello"`, `` `hi ${name}` `` |
| `int`    | `42`                   |
| `float`  | `3.14` *(if enabled)*  |
| `array`  | `[1, 2, 3]`            |
| `map`    | `{ key: "value" }`     |
| `fn`     | `fn name(x) { ... }`   |

## ➕ Operators

| Operator | Description         | Example                |
|----------|---------------------|------------------------|
| `+`      | Add / Concatenate   | `2 + 3`, `"a" + "b"`   |
| `-`      | Subtract            | `10 - 4`               |
| `*`      | Multiply            | `5 * 6`                |
| `/`      | Divide              | `9 / 3`                |
| `%`      | Modulo              | `10 % 3`               |
| `==`     | Equals              | `x == y`               |
| `!=`     | Not equal           | `x != y`               |
| `<`, `>`, `<=`, `>=` | Comparisons | `x >= 10`        |
| `&&`     | Logical AND         | `x && y`               |
| `||`     | Logical OR          | `x || y`               |
| `!`      | Logical NOT         | `!x`                   |

## 🌐 Global Scope

- `main` – the full YAML input
- Declared `modules` – globally exposed
- Utility functions like `log(...)`

## 🧪 Expressions & Statements

```rust
let upper = main.name.to_uppercase().trim();
```

## 🔀 Ternary Expressions

```rust
let msg = main.name == "" ? "Anonymous" : `Hello, ${main.name}`;
```

## 🔎 Type Conversion Helpers

```rust
let number = "42".to_int();
let flag = "true".to_bool();
```

## 🛠 Working with Maps & Arrays

```rust
let keys = user.keys();
let vals = user.values();
if fruits.contains("banana") {
  log("info", "Found it!");
}
```

## 🧯 Error Handling

Structured try/catch is not supported.

## 🪛 Debugging Tools

```rust
log("debug", `Debugging var: ${data}`);
```

## 🧬 Nested Access in YAML

```yaml
config:
  retries: 3
  labels:
    - core
    - beta
```

```rust
let retry = main.config.retries;
let tag = main.config.labels[0];
```

## 📍Future Support Notes

- `break` / `continue` → *not supported yet*
- `match` / pattern matching → *planned*
- `try/catch` → *TBD*
