---
sidebar_position: 4
title: Running
hide_title: true
---
#  Running a Flow

By default, Phlow will look for a \`main.phlow\` in the current directory:

```bash
phlow
```

To run a specific file:

```bash
phlow path/to/your-flow.phlow
```

If you provide a directory path and it contains a \`main.phlow\`, Phlow will automatically run that:

```bash
phlow path/to/directory
# → runs path/to/directory/main.phlow
```

###  Help

For all available options and usage info:

```bash
phlow -h
# or
phlow --help
```
