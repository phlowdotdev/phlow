<p align="center"> <img src="../site/phlow.svg" alt="Phlow logo" width="140"/> </p> <h1 align="center">PHS – Phlow Script</h1>


**PHS** is a lightweight scripting format for [Phlow](https://github.com/phlowdotdev/phlow), built on top of [Rhai](https://rhai.rs/). It enables simple, dynamic behavior scripting using `.phs` files while deeply integrating with the Phlow runtime and module system.

## ✨ Overview

PHS (Phlow Script) brings the power of embedded scripting to YAML-based workflows. It's designed to let you inject dynamic logic through readable scripts, while preserving Phlow's declarative style.

You can inject modules directly into your PHS context via the `modules` section of your `.yaml` configuration. Each module declared becomes globally accessible in the `.phs` script, making it easy to mix scripting with orchestrated steps.

## 📑 Summary

- [✨ Overview](#-overview)
- [🔌 Module Injection via YAML](#-module-injection-via-yaml)
- [🧪 Example](#-example)
  - [main.phlow](#mainphlow)
  - [script.phs](#scriptphs)
  - [💡Output](#output)
- [📁 File Extensions](#-file-extensions)
- [🔐 Modules Supported in PHS](#-modules-supported-in-phs)
- [🧠 Variables in PHS](#-variables-in-phs)
  - [🔤 Declaring Variables](#-declaring-variables)
  - [✍️ Reassigning Values](#️-reassigning-values)
  - [🔄 Using Function Results](#-using-function-results)
- [🧱 Arrays and Objects (Maps)](#-arrays-and-objects-maps)
  - [📚 Arrays](#-arrays)
  - [🔄 Looping Through Arrays](#-looping-through-arrays)
  - [🧳 Objects (Maps)](#-objects-maps)
  - [📦 Nesting](#-nesting)
- [🧭 Conditionals in PHS](#-conditionals-in-phs)
  - [✅ Basic If](#-basic-if)
  - [🔁 If...Else](#-ifelse)
  - [🔀 Else If](#-else-if)
  - [🔗 Nested Conditions](#-nested-conditions)
- [🔁 Loops in PHS](#-loops-in-phs)
  - [📚 Looping Through an Array](#-looping-through-an-array)
  - [🔢 Looping with a Range](#-looping-with-a-range)
  - [🔄 Nested Loops](#-nested-loops)
  - [🛑 Breaking a Loop (not supported yet)](#-breaking-a-loop-not-supported-yet)
- [🧩 Functions in PHS](#-functions-in-phs)
  - [🛠 Defining a Function](#-defining-a-function)
  - [▶️ Calling a Function](#️-calling-a-function)
  - [↩️ Returning Values](#️-returning-values)
  - [🧠 Functions with Logic](#-functions-with-logic)
  - [⚠️ Scope](#️-scope)
- [🧬 PHS Syntax and Language Features](#-phs-syntax-and-language-features)
  - [📐 Data Types in PHS](#-data-types-in-phs)
  - [➕ Operators](#-operators)
  - [🌐 Global Scope](#-global-scope)
  - [🧪 Expressions & Statements](#-expressions--statements)
  - [🔀 Ternary Expressions](#-ternary-expressions)
  - [� String Functions](#-string-functions)
  - [�🔎 Type Conversion Helpers](#-type-conversion-helpers)
  - [🛠 Working with Maps & Arrays](#-working-with-maps--arrays)
  - [🧯 Error Handling](#-error-handling)
  - [🪛 Debugging Tools](#-debugging-tools)
  - [🧬 Nested Access in YAML](#-nested-access-in-yaml)
  - [📍Future Support Notes](#future-support-notes)

## 🔌 Module Injection via YAML

All modules declared in the YAML under `modules:` are automatically available inside your `.phs` script. For example, when you load the `log` module, its functions can be used directly in the script.

## 🧪 Example
#### main.phlow

```yaml
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

#### script.phs
```rust
log("warn", `Hello, ${main.name}`);
```

### 💡Output
If the user runs:
```bash
phlow run main.phlow --name Philippe
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
## 🧱 Arrays and objects (maps)
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
| `null`    | `null`                |
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

PHS supports ternary expressions using the `when` keyword for conditional logic:

```rust
let msg = when main.name == "" ? "Anonymous" : `Hello, ${main.name}`;
let status = when age >= 18 ? "adult" : "minor";
let value = when condition ? true_value : false_value;
```

## 🔤 String Functions

PHS includes several custom string manipulation functions in addition to Rhai's built-in string methods:

### 🔍 `search(pattern)` - Regex Pattern Matching
Search for regex patterns in strings, returns `true` if found:

```rust
let text = "Hello World";
let hasHello = text.search("Hello");        // true
let startsWithH = text.search("^H");        // true (regex: starts with H)
let endsWithD = text.search("d$");          // true (regex: ends with d)
let hasNumbers = text.search("[0-9]");      // false
```

### 🔄 `replace(target, replacement)` - String Replacement
⚠️ **Important:** Unlike native Rhai `replace`, this function **returns** the modified string instead of changing the variable in place:

```rust
let text = "Hello World";
let newText = text.replace("World", "Universe");  // Returns "Hello Universe"
// text is still "Hello World" - original unchanged
```

### ✂️ `slice(start, end)` - Substring Extraction
Extract a portion of a string with bounds checking:

```rust
let text = "Hello World";
let part = text.slice(0, 5);     // "Hello"
let middle = text.slice(6, 11);  // "World"
let safe = text.slice(0, 100);   // "Hello World" (auto-bounds)
```

### 🎩 `capitalize()` - First Letter Uppercase
Capitalize the first character of a string:

```rust
let name = "joão";
let capitalized = name.capitalize();  // "João"
let empty = "".capitalize();         // ""
```

### 🐍 Case Conversion Functions
Convert between different naming conventions. These functions automatically detect the current format:

```rust
// to_snake_case() - Convert to snake_case
"meuTextoExemplo".to_snake_case();    // "meu_texto_exemplo"
"MeuTextoExemplo".to_snake_case();    // "meu_texto_exemplo"  
"meu-texto-exemplo".to_snake_case();  // "meu_texto_exemplo"
"Meu texto exemplo".to_snake_case();  // "meu_texto_exemplo"

// to_camel_case() - Convert to camelCase
"meu_texto_exemplo".to_camel_case();  // "meuTextoExemplo"
"meu-texto-exemplo".to_camel_case();  // "meuTextoExemplo"
"Meu texto exemplo".to_camel_case();  // "meuTextoExemplo"

// to_kebab_case() - Convert to kebab-case
"meuTextoExemplo".to_kebab_case();    // "meu-texto-exemplo"
"meu_texto_exemplo".to_kebab_case();  // "meu-texto-exemplo"
"Meu texto exemplo".to_kebab_case();  // "meu-texto-exemplo"
```

### 📖 Additional String Methods
For more string manipulation functions, refer to [Rhai Language Reference](https://rhai.rs/book/ref/index.html).

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
