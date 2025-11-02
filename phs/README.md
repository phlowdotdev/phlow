<p align="center">
  <img src="../site/phlow.svg" alt="phlow logo" width="140"/>
</p>

# PHS – Phlow Script

**PHS** (Phlow Script) é um formato de script leve para [Phlow](https://github.com/phlowdotdev/phlow), baseado em [Rhai](https://rhai.rs/). Permite lógica dinâmica, manipulação de dados e integração profunda com módulos Phlow, tudo em arquivos `.phs`.

## ✨ Visão Geral

PHS traz o poder do scripting embutido para workflows YAML, permitindo lógica dinâmica, manipulação de variáveis, funções, arrays, objetos e integração com módulos Rust customizados.

Scripts `.phs` podem ser importados em flows Phlow via `!import`, e têm acesso global aos módulos declarados no YAML.

## 📑 Sumário

- [✨ Visão Geral](#-visão-geral)
- [🔌 Injeção de Módulos via YAML](#-injeção-de-módulos-via-yaml)
- [🧪 Exemplo](#-exemplo)
- [📁 Extensões de Arquivo](#-extensões-de-arquivo)
- [🔐 Módulos Suportados](#-módulos-suportados)
- [🧠 Variáveis](#-variáveis)
- [🧱 Arrays e Objetos (Maps)](#-arrays-e-objetos-maps)
- [🧭 Condicionais](#-condicionais)
- [🔁 Loops](#-loops)
- [🧩 Funções](#-funções)
- [🧬 Sintaxe e Recursos](#-sintaxe-e-recursos)

## 🔌 Injeção de Módulos via YAML

Todos os módulos declarados em `modules:` no YAML ficam disponíveis globalmente no script `.phs`.

## 🧪 Exemplo
### main.phlow
```yaml
main: cli
modules:
  - module: cli
  - module: log
steps:
  - return: !import script.phs
```
### script.phs
```rust
log("warn", `Hello, ${main.name}`);
// or
log({
  level: "warn",
  message: `Hello, ${main.name}`
});
```

## 📁 Extensões de Arquivo
Phlow carrega scripts `.phs` via `!import` e executa com engine Rhai estendida.

## � Módulos Suportados
Qualquer módulo com bindings de scripting pode ser usado: log, cli, http_server, etc.

## 🧠 Variáveis
Declare variáveis com `let`:
```rust
let nome = main.name;
let saudacao = "Olá";
let mensagem = `${saudacao}, ${nome}!`;
log("info", mensagem);
```
Reatribuição:
```rust
let cont = 1;
cont = cont + 1;
```
Funções podem retornar valores:
```rust
let status = "warn";
let msg = "Algo aconteceu";
log(status, msg);
```

## 🧱 Arrays e Objetos (Maps)
Arrays:
```rust
let frutas = ["maçã", "banana", "laranja"];
frutas.push("uva");
```
Objetos:
```rust
let usuario = #{ nome: main.name, idade: 30 };
usuario.idade = 31;
usuario.status = "online";
```
Nesting:
```rust
let config = #{ tags: ["dev"], options: #{ retries: 3 } };
```

## 🧭 Condicionais
```rust
if main.name == "Philippe" {
  log("info", "Bem-vindo!");
} else if main.name == "Alice" {
  log("info", "Oi Alice!");
} else {
  log("info", "Olá, visitante!");
}
```

## 🔁 Loops
```rust
for fruta in frutas {
  log("info", `Fruta: ${fruta}`);
}
for i in 0..5 {
  log("debug", `Índice: ${i}`);
}
```

## 🧩 Funções
Defina funções com `fn`:
```rust
fn saudacao(nome) {
  log("info", `Olá, ${nome}!");
}
saudacao("Philippe");
```
Funções podem retornar valores:
```rust
fn dobro(n) { return n * 2; }
let resultado = dobro(5);
```

## 🧬 Sintaxe e Recursos
Tipos suportados: bool, string, int, float, array, null, map, fn
Operadores: +, -, *, /, %, ==, !=, <, >, <=, >=, &&, ||, !
Escopo global: main, modules, funções utilitárias

### Expressões & Ternário
```rust
let msg = when main.name == "" ? "Anônimo" : `Olá, ${main.name}`;
```

### Funções de String
- `search(pattern)` — regex
- `starts_with(prefix)` — prefixo
- `replace(target, replacement)` — substituição
- `slice(start, end)` / `slice(start)` — substring
- `capitalize()` — primeira letra maiúscula
- `to_snake_case()`, `to_camel_case()`, `to_kebab_case()` — conversão de case
- `to_url_encode()` — codificação URL
- `to_base64()` — codificação Base64
- `base64_to_utf8()` — decodificação Base64
- `url_decode()` — decodificação URL

### Conversão de Tipos
```rust
let numero = "42".to_int();
let flag = "true".to_bool();
```

### Manipulação de Maps & Arrays
```rust
let chaves = usuario.keys();
if frutas.contains("banana") { log("info", "Achou!"); }
```

### Debug
```rust
log("debug", `Debug: ${data}`);
```

### Acesso aninhado YAML
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

### Notas Futuras
- `break` / `continue` — não suportado
- `match` — planejado
- `try/catch` — TBD

---
Para mais detalhes, consulte a documentação oficial do Phlow e do Rhai.
- **Standard encoding:** Uses the standard Base64 alphabet
- **Automatic padding:** Adds `=` characters when needed
- **UTF-8 support:** Handles special characters correctly
- **Binary safe:** Works with any byte sequence

### � `base64_to_utf8()` - Base64 Decoding

Decode Base64 strings back to UTF-8 text:

```rust
"SGVsbG8gV29ybGQ=".base64_to_utf8();        // "Hello World"
"dXNlckBleGFtcGxlLmNvbQ==".base64_to_utf8(); // "user@example.com"
"Y2Fmw6k=".base64_to_utf8();                // "café"
"MTIzNDU=".base64_to_utf8();                // "12345"
"".base64_to_utf8();                        // ""
"invalid_base64!@#".base64_to_utf8();       // "" (empty on error)
```

**Features:**
- **Standard decoding:** Uses the standard Base64 alphabet
- **UTF-8 validation:** Returns empty string if result is not valid UTF-8
- **Error handling:** Returns empty string for invalid Base64 input
- **Safe operation:** Never crashes on malformed input

### 🔓 `url_decode()` - URL Decoding

Decode URL-encoded strings back to UTF-8 text:

```rust
"Hello+World".url_decode();              // "Hello World"
"user%40example.com".url_decode();       // "user@example.com"
"caf%C3%A9+%26+ma%C3%A7%C3%A3".url_decode(); // "café & maçã"
"abc-123_test.file~".url_decode();       // "abc-123_test.file~" (unchanged)
"Ol%C3%A1+mundo%21".url_decode();        // "Olá mundo!"
"%ZZ".url_decode();                      // "%ZZ" (invalid hex preserved)
"test%".url_decode();                    // "test%" (incomplete sequence preserved)
```

**Features:**
- **RFC 3986 compliant:** Handles standard URL encoding rules
- **Plus to space:** Converts `+` characters to spaces
- **UTF-8 support:** Properly decodes multi-byte UTF-8 sequences
- **Error tolerance:** Preserves malformed sequences rather than failing
- **Safe operation:** Returns empty string only for UTF-8 validation errors

### �📋 `parse()` - JSON Parser

Parse JSON strings into native Rhai types:

```rust
"\"hello world\"".parse();    // "hello world" (string)
"42".parse();                 // 42 (integer)
"3.14".parse();               // 3.14 (float)
"true".parse();               // true (boolean)
"false".parse();              // false (boolean)
"null".parse();               // () (unit/null)

// Objects are converted to Rhai Maps
let obj = "{\"name\":\"João\",\"age\":30}".parse();
obj.name;                     // "João"
obj.age;                      // 30

// Arrays are converted to Rhai Arrays
let arr = "[1, 2, 3, \"test\"]".parse();
arr[0];                       // 1
arr[3];                       // "test"
arr.len();                    // 4

// Nested structures work too
let nested = "{\"user\":{\"name\":\"Maria\",\"roles\":[\"admin\",\"user\"]}}".parse();
nested.user.name;             // "Maria"
nested.user.roles[0];         // "admin"
```

**Features:**
- **Type conversion:** Automatically converts to appropriate Rhai types
- **Primitive support:** Handles strings, numbers, booleans, and null
- **Native structures:** Objects become Maps, arrays become Arrays
- **Nested support:** Handles complex nested JSON structures
- **Direct access:** Use dot notation and indexing on parsed objects/arrays
- **Error handling:** Returns null (unit) for invalid JSON
- **Safe parsing:** Never crashes on malformed input

### 📖 Additional String Methodsehavior scripting using `.phs` files while deeply integrating with the Phlow runtime and module system.

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

### 🎯 `starts_with(prefix)` - Prefix Checking
Check if a string starts with a specific prefix:

```rust
let auth = "Bearer abc123";
let hasBearer = auth.starts_with("Bearer");     // true
let hasSpace = auth.starts_with("Bearer ");     // true
let isBasic = auth.starts_with("Basic");        // false

let email = "user@example.com";
let isUserEmail = email.starts_with("user");   // true
let hasAt = email.starts_with("@");             // false

let empty = "";
let anyString = "test".starts_with("");         // true (empty prefix always matches)
```

**Features:**
- **Case-sensitive:** Distinguishes between uppercase and lowercase
- **Exact matching:** No regex patterns, just literal string comparison
- **Fast operation:** More efficient than regex for simple prefix checks
- **Empty prefix:** Always returns `true` for empty string prefix
- **Safe operation:** Never fails, returns `false` for invalid cases

**When to use `starts_with()` vs `search()`:**
- Use `starts_with("prefix")` for simple prefix checking (faster)
- Use `search("^prefix")` for regex-based prefix checking with patterns

### 🔄 `replace(target, replacement)` - String Replacement
⚠️ **Important:** Unlike native Rhai `replace`, this function **returns** the modified string instead of changing the variable in place:

```rust
let text = "Hello World";
let newText = text.replace("World", "Universe");  // Returns "Hello Universe"
// text is still "Hello World" - original unchanged
```

### ✂️ `slice(start, end)` / `slice(start)` - Substring Extraction
Extract a portion of a string with bounds checking. Supports two variants:

**Two parameters - `slice(start, end)`:**
```rust
let text = "Hello World";
let part = text.slice(0, 5);     // "Hello"
let middle = text.slice(6, 11);  // "World"
let safe = text.slice(0, 100);   // "Hello World" (auto-bounds)
```

**One parameter - `slice(start)`:** 
```rust
let text = "abcdef";
let fromIndex = text.slice(3);    // "def" (from index 3 to end)
let lastTwo = text.slice(-2);     // "ef" (last 2 characters)
let fromStart = text.slice(0);    // "abcdef" (entire string)
```

- **Positive index:** Takes from that position to the end
- **Negative index:** Takes the last N characters

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

### � `to_url_encode()` - URL Encoding
Encode strings for safe use in URLs following RFC 3986:

```rust
"Hello World".to_url_encode();        // "Hello+World"
"user@example.com".to_url_encode();   // "user%40example.com"
"café & maçã".to_url_encode();        // "caf%C3%A9+%26+ma%C3%A7%C3%A3"
"abc-123_test.file~".to_url_encode(); // "abc-123_test.file~" (unchanged)
```

**Encoding rules:**
- **Safe characters:** Letters, numbers, `-`, `_`, `.`, `~` remain unchanged
- **Spaces:** Converted to `+`
- **Other characters:** Encoded as `%XX` (hexadecimal)
- **UTF-8:** Full support for multi-byte characters

### 🔐 `to_base64()` - Base64 Encoding
Encode strings to Base64 format (RFC 4648):

```rust
"Hello World".to_base64();      // "SGVsbG8gV29ybGQ="
"user@example.com".to_base64(); // "dXNlckBleGFtcGxlLmNvbQ=="
"café".to_base64();             // "Y2Fmw6k="
"12345".to_base64();            // "MTIzNDU="
"".to_base64();                 // ""
```

**Features:**
- **Standard encoding:** Uses the standard Base64 alphabet
- **Automatic padding:** Adds `=` characters when needed
- **UTF-8 support:** Handles special characters correctly
- **Binary safe:** Works with any byte sequence

### �📖 Additional String Methods
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
