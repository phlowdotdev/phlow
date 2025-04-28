---
sidebar_position: 6
title: Phlow Scripts (.phs)
---

**Phlow Scripts (PHS)** is a lightweight scripting format for [Phlow](https://github.com/phlowdotdev/phlow), built on top of [Rhai](https://rhai.rs/). It enables simple, dynamic behavior scripting using `.phs` files while deeply integrating with the Phlow runtime and module system.

## ✨ Overview

PHS (Phlow Script) brings the power of embedded scripting to YAML-based workflows. It's designed to let you inject dynamic logic through readable scripts, while preserving Phlow's declarative style.

You can inject modules directly into your PHS context via the `modules` section of your `.yaml` configuration. Each module declared becomes globally accessible in the `.phs` script, making it easy to mix scripting with orchestrated steps.

## 📚 Learn More

For a comprehensive guide on Phlow Scripts, visit the [official documentation](https://github.com/phlowdotdev/phlow/tree/main/phs#readme).