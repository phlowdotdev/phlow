---
sidebar_position: 2
title: CLI - Are you a student?
---
This is a simple example of a Phlow that uses the `cli` module to ask the user if they are a student. It uses the `assert` step to check the user's input and return different messages based on their response.


```yaml
main: cli
name: Are you a student?
version: 1.0.0
description: Check if you are a student.
author: Your Name
modules: 
  - module: cli
    # version: latest (optional - defaults to latest)
    with:
      additional_args: false
      args: 
        - name: name
          description: Student name
          index: 1
          type: string
          required: true
        - name: age
          description: Student age
          index: 2
          type: number
          required: true
        - name: force
          long: force
          description: Force assertion
          short: f
          type: boolean
          default: false
steps:
  - assert: !phs main.force
    then:
      return: !phs `${main.name} is a student, but the age is not valid`
    else:
      - assert: !phs main.age < 18 && main.age > 3
        then:
          return: !phs `${main.name} is a student`
      - assert: !phs main.age >= 18
        then:
          return: !phs `${main.name} is not a student`
      - assert: !phs main.age <= 3
        then:
          return: !phs `${main.name} is a baby`  
      - return: !phs `${main.name} is not a valid age`
```

### Run the Phlow
You can run the Phlow using the command line. By default, Phlow will look for a `main.yaml` in the current directory:

```bash
phlow main.yaml
```
### Test
You can test the Phlow by running the command with different arguments. For example, to check if you are a student, you can run:

```bash
phlow main.yaml John 20
```
This command will output:

```bash
John is not a student
```
### Expected Output
When you run the command, the Phlow will check the age and return the appropriate message. For example, if you run:

```bash
phlow main.yaml John 20
```
The output will be:

```bash
John is not a student
```
If you run:

```bash
phlow main.yaml John 15
```
The output will be:

```bash
John is a student
```
If you run:

```bash
phlow main.yaml John 2
```
The output will be:

```bash
John is a baby
```
If you run:

```bash
phlow main.yaml John 25 --force
```
The output will be:

```bash
John is a student, but the age is not valid
```
### Conclusion
This example demonstrates how to use the `cli` module in Phlow to create a simple command-line application that checks if a user is a student based on their age. You can customize the Phlow further by adding more steps or modifying the existing ones.

### Notes
- The `assert` step is used to check conditions and return different messages based on the user's input.
- The `then` and `else` clauses are used to define the actions to take based on the assertions.
- The `return` step is used to return the final message based on the assertions.
