---
sidebar_positiYou don't need to worry about building or installing them manually — just describe the modules in your Phlow files, and Phlow takes care of the rest.n: 1
title: Introduction
hide_title: true
---

#  Packages and Modules

Phlow has a powerful package management system that allows you to import and use third-party modules in your workflows. This makes it easier to reuse code and integrate with external libraries.

## Automatic Module Download

Phlow automatically downloads the modules specified in your flow configuration.

The official module repository is [phlow-packages](https://github.com/phlowdotdev/phlow-packages), which contains all official Phlow modules precompiled for Linux.

When you run Phlow, it will automatically fetch and install the required modules into a local `phlow-packages/` folder at the root of your project execution.

You don’t need to worry about building or installing them manually — just describe the modules in your YAML, and Phlow takes care of the rest.

## Using modules

To use a module in your flow, you only need to declare it under the `modules` section and reference it in your `steps`.

Here’s a minimal working example that uses the official `log` module:

```phlow
main: log_example
modules:
  - module: log
    version: latest
steps:
  - module: log
    input:
      level: info
      message: "📥 Starting process..."
  - module: log
    input:
      level: debug
      message: !phs "'Current time: ' + timestamp()"
  - module: log
    input:
      level: error
      message: "❌ Something went wrong"
```