---
sidebar_position: 4
title: Steps
---
Steps control the logic and flow of execution.

Example:

```yaml
steps:
  - assert: !phs main.force
    then:
      return: !phs `${main.name} is a student, but the age is not valid`
    else:
      # Additional conditional steps
```

Each step may have:

- `assert`: Evaluates a condition.
- `then`: Executes if the assertion is true.
- `else`: Executes if the assertion is false.

## All Elements
### id
A unique identifier for the step, used for tracking or referencing. This allows the step's output to be accessed using the `steps` variable. For example:

```yaml
steps:
  - id: my_step
    payload:
      key: value
  - assert: !phs steps.my_step.payload.key == 'value'
    then:
      return: !phs `Step ID is valid and payload key matches`
```
### label
A descriptive label for the step, often used as the span name in OpenTelemetry for tracing and monitoring purposes. This label helps in identifying and analyzing the step during distributed tracing. For example:

```yaml
steps:
  - label: Validate User Age
    assert: !phs main.age > 18
    then:
      return: !phs `User is an adult`
    else:
      return: !phs `User is underage`
```
### assert
Evaluates a boolean expression to control the flow. If the `assert` is true, it executes the `then` block. If false, it executes the `else` block, if defined. If the `else` block is not present, the flow proceeds to the next step. For example:

```yaml
steps:
  - assert: !phs main.age > 18
    then:
      return: !phs `User is an adult`
    else:
      return: !phs `User is underage`
```
### payload
Represents the data that the step sends forward and can also receive from the previous step. By declaring `payload`, you define the data to be passed to the next step. The subsequent step can capture it using `!phs payload`. For example:

```yaml
steps:
  - payload:
      key: value
  - assert: !phs payload.key == 'value'
    then:
      return: !phs `Payload key is valid`
```
### use
Specifies the context in which the step is executed. This is used to invoke specific modules that perform specialized tasks, such as logging, producing messages to an AMQP queue, querying a database, or other operations that are not part of the main context. 

For example:

1. **Using the `log` module**:  
   Logs a message for tracing or debugging purposes.
   ```yaml
   steps:
     - use: log
       input:
         message: !phs `Logging a message for tracing`
   ```

2. **Using the `producer` module**:  
   Sends a message to an AMQP queue.
   ```yaml
   steps:
     - use: producer
       input:
         queue: "task_queue"
         message: "Task data to process"
   ```

3. **Using the `query` module**:  
   Executes a SQL query on a PostgreSQL database.
   ```yaml
   steps:
     - use: postgres
       input:
         query: "SELECT * FROM users WHERE active = true"
   ```

### to
The to represents the ID of a step that is identified as the next step to be executed in sequence, effectively linking the current step to the subsequent one.

  ```yaml
  steps:
    - to: step4
    - id: step2
      payload: !phs payload + 1
    - assert: !phs payload == 2
      then:
        return: !phs `Payload is 2`
      else:
        return: !phs `Payload is not 2`
    - id: step3
      payload: 1
      to: step2
  ```

### steps
It is possible to use `steps` to execute a sequence of steps within the context of another step. This is particularly useful in scenarios where you want to define additional logic inside `then` or `else` blocks. For example:

```yaml
steps:
  - assert: !phs main.age > 18
    then:
      label: Check Adult Status
      steps:
        - use: log
          input:
            message: "User is an adult"
        - return: !phs `User is an adult`
    else:
      label: Check Underage Status
      steps:
        - use: log
          input:
            message: "User is underage"
        - return: !phs `User is underage`
```
### return
Similar to the `return` statement in any programming language, it halts the flow of execution and returns the specified data to the main context. For example:

```yaml
steps:
  - assert: !phs main.age > 18
    then:
      return: !phs `User is an adult`
    else:
      return: !phs `User is underage`
```