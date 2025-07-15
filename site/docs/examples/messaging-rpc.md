---
sidebar_position: 5
title: Messaging & RPC
---

# Messaging & RPC

Phlow supports asynchronous messaging through AMQP (RabbitMQ) and RPC (Remote Procedure Call) patterns for building distributed systems.

## Basic AMQP Producer

Send messages to a RabbitMQ queue:

```yaml title="amqp-producer.phlow"
name: AMQP Producer
version: 1.0.0
description: Send messages to RabbitMQ queue

modules:
  - module: amqp
    version: latest
    with:
      host: localhost
      port: 5672
      username: guest
      password: guest
      vhost: /

steps:
  - payload: !phs {
      messages: [
        { id: 1, text: "Hello from Phlow!", priority: "high" },
        { id: 2, text: "Processing order #1234", priority: "medium" },
        { id: 3, text: "User registration completed", priority: "low" }
      ]
    }
  
  # Send each message
  - payload: !phs {
      ...payload,
      sent_messages: []
    }
  
  # Send message 1
  - amqp:
      action: publish
      exchange: ""
      routing_key: "task_queue"
      message: !phs JSON.stringify(payload.messages[0])
      properties:
        delivery_mode: 2  # Make message persistent
        priority: 3
  
  - payload: !phs {
      ...payload,
      sent_messages: [...payload.sent_messages, payload.messages[0]]
    }
  
  # Send message 2
  - amqp:
      action: publish
      exchange: ""
      routing_key: "task_queue"
      message: !phs JSON.stringify(payload.messages[1])
      properties:
        delivery_mode: 2
        priority: 2
  
  - payload: !phs {
      ...payload,
      sent_messages: [...payload.sent_messages, payload.messages[1]]
    }
  
  # Send message 3
  - amqp:
      action: publish
      exchange: ""
      routing_key: "task_queue"
      message: !phs JSON.stringify(payload.messages[2])
      properties:
        delivery_mode: 2
        priority: 1
  
  - payload: !phs {
      ...payload,
      sent_messages: [...payload.sent_messages, payload.messages[2]]
    }
  
  - return: !phs {
      total_sent: payload.sent_messages.length,
      messages: payload.sent_messages
    }
```

## AMQP Consumer

Consume messages from a RabbitMQ queue:

```yaml title="amqp-consumer.phlow"
name: AMQP Consumer
version: 1.0.0
description: Consume messages from RabbitMQ queue

modules:
  - module: amqp
    version: latest
    with:
      host: localhost
      port: 5672
      username: guest
      password: guest
      vhost: /

steps:
  # Declare queue
  - amqp:
      action: declare_queue
      queue: "task_queue"
      durable: true
  
  # Consume messages
  - amqp:
      action: consume
      queue: "task_queue"
      auto_ack: false
      prefetch_count: 1
  
  # Process message
  - payload: !phs {
      received_message: JSON.parse(payload.message),
      delivery_tag: payload.delivery_tag,
      processed_at: new Date().toISOString()
    }
  
  - log:
      message: !phs `Processing message: ${payload.received_message.text}`
  
  # Simulate processing time based on priority
  - sleep:
      seconds: !phs payload.received_message.priority === "high" ? 1 : (payload.received_message.priority === "medium" ? 2 : 3)
  
  # Acknowledge message
  - amqp:
      action: ack
      delivery_tag: !phs payload.delivery_tag
  
  - return: !phs {
      processed: payload.received_message,
      processing_time: payload.received_message.priority === "high" ? 1 : (payload.received_message.priority === "medium" ? 2 : 3),
      processed_at: payload.processed_at
    }
```

## RPC Server

Create an RPC server that handles remote procedure calls:

```yaml title="rpc-server.phlow"
name: RPC Server
version: 1.0.0
description: Handle RPC calls for mathematical operations

modules:
  - module: rpc
    version: latest
    with:
      host: localhost
      port: 8080
      methods:
        - name: add
          description: Add two numbers
        - name: multiply
          description: Multiply two numbers
        - name: factorial
          description: Calculate factorial
        - name: fibonacci
          description: Calculate Fibonacci number

steps:
  - payload: !phs {
      method: main.method,
      params: main.params,
      request_id: main.request_id
    }
  
  # Handle add method
  - assert: !phs payload.method == "add"
    then:
      - payload: !phs {
          ...payload,
          result: payload.params.a + payload.params.b
        }
      - return: !phs {
          jsonrpc: "2.0",
          result: payload.result,
          id: payload.request_id
        }
  
  # Handle multiply method
  - assert: !phs payload.method == "multiply"
    then:
      - payload: !phs {
          ...payload,
          result: payload.params.a * payload.params.b
        }
      - return: !phs {
          jsonrpc: "2.0",
          result: payload.result,
          id: payload.request_id
        }
  
  # Handle factorial method
  - assert: !phs payload.method == "factorial"
    then:
      - payload: !phs {
          ...payload,
          n: payload.params.n,
          result: 1
        }
      - payload: !phs {
          ...payload,
          result: Array.from({length: payload.n}, (_, i) => i + 1)
            .reduce((acc, val) => acc * val, 1)
        }
      - return: !phs {
          jsonrpc: "2.0",
          result: payload.result,
          id: payload.request_id
        }
  
  # Handle fibonacci method
  - assert: !phs payload.method == "fibonacci"
    then:
      - payload: !phs {
          ...payload,
          n: payload.params.n
        }
      - payload: !phs {
          ...payload,
          result: payload.n <= 1 ? payload.n : 
            (() => {
              let a = 0, b = 1;
              for (let i = 2; i <= payload.n; i++) {
                [a, b] = [b, a + b];
              }
              return b;
            })()
        }
      - return: !phs {
          jsonrpc: "2.0",
          result: payload.result,
          id: payload.request_id
        }
  
  # Handle unknown method
  - return: !phs {
      jsonrpc: "2.0",
      error: {
        code: -32601,
        message: "Method not found"
      },
      id: payload.request_id
    }
```

## RPC Client

Create an RPC client to call remote procedures:

```yaml title="rpc-client.phlow"
name: RPC Client
version: 1.0.0
description: Call remote procedures

main: cli
modules:
  - module: cli
    version: latest
    with:
      args:
        - name: method
          description: RPC method to call
          index: 1
          type: string
          required: true
          choices: ["add", "multiply", "factorial", "fibonacci"]
        - name: params
          description: JSON parameters for the method
          index: 2
          type: string
          required: true
        - name: server_url
          long: server
          short: s
          description: RPC server URL
          type: string
          default: "http://localhost:8080/rpc"
  - module: rpc
    version: latest
  - module: log
    version: latest

steps:
  - payload: !phs {
      method: main.method,
      params: JSON.parse(main.params),
      server_url: main.server_url,
      request_id: Math.random().toString(36).substr(2, 9)
    }
  
  - log:
      message: !phs `Calling ${payload.method} with params: ${JSON.stringify(payload.params)}`
  
  # Make RPC call
  - rpc:
      url: !phs payload.server_url
      method: !phs payload.method
      params: !phs payload.params
      id: !phs payload.request_id
  
  - log:
      message: !phs `Response: ${JSON.stringify(payload)}`
  
  - return: !phs payload
```

### Usage Examples

```bash
# Add two numbers
phlow rpc-client.phlow add '{"a": 5, "b": 3}'

# Multiply numbers
phlow rpc-client.phlow multiply '{"a": 4, "b": 6}'

# Calculate factorial
phlow rpc-client.phlow factorial '{"n": 5}'

# Calculate Fibonacci
phlow rpc-client.phlow fibonacci '{"n": 10}'
```

## Work Queue Pattern

Implement a work queue for distributed task processing:

```yaml title="work-queue.phlow"
name: Work Queue
version: 1.0.0
description: Distribute tasks across multiple workers

main: cli
modules:
  - module: cli
    version: latest
    with:
      args:
        - name: role
          description: Role to run (producer or consumer)
          index: 1
          type: string
          required: true
          choices: ["producer", "consumer"]
        - name: task_count
          description: Number of tasks to produce
          index: 2
          type: number
          default: 10
  - module: amqp
    version: latest
    with:
      host: localhost
      port: 5672
      username: guest
      password: guest
      vhost: /

steps:
  - payload: !phs {
      role: main.role,
      task_count: main.task_count || 10
    }
  
  # Producer role
  - assert: !phs payload.role == "producer"
    then:
      - log:
          message: !phs `Starting producer - will create ${payload.task_count} tasks`
      
      # Declare queue
      - amqp:
          action: declare_queue
          queue: "work_queue"
          durable: true
      
      # Create tasks
      - payload: !phs {
          ...payload,
          tasks: Array.from({length: payload.task_count}, (_, i) => ({
            id: i + 1,
            task_type: ["email", "image_processing", "data_analysis"][Math.floor(Math.random() * 3)],
            complexity: Math.floor(Math.random() * 5) + 1,
            created_at: new Date().toISOString()
          }))
        }
      
      # Send tasks to queue
      - payload: !phs {
          ...payload,
          sent_tasks: []
        }
      
      # Send each task (simplified - in real implementation, you'd loop)
      - amqp:
          action: publish
          exchange: ""
          routing_key: "work_queue"
          message: !phs JSON.stringify(payload.tasks[0])
          properties:
            delivery_mode: 2
      
      - return: !phs {
          message: `Successfully sent ${payload.tasks.length} tasks to queue`,
          tasks: payload.tasks
        }
  
  # Consumer role
  - assert: !phs payload.role == "consumer"
    then:
      - log:
          message: "Starting consumer - waiting for tasks"
      
      # Declare queue
      - amqp:
          action: declare_queue
          queue: "work_queue"
          durable: true
      
      # Consume task
      - amqp:
          action: consume
          queue: "work_queue"
          auto_ack: false
          prefetch_count: 1
      
      # Process task
      - payload: !phs {
          task: JSON.parse(payload.message),
          delivery_tag: payload.delivery_tag,
          worker_id: `worker_${Math.random().toString(36).substr(2, 5)}`
        }
      
      - log:
          message: !phs `${payload.worker_id} processing task ${payload.task.id}: ${payload.task.task_type}`
      
      # Simulate work based on complexity
      - sleep:
          seconds: !phs payload.task.complexity
      
      # Acknowledge task completion
      - amqp:
          action: ack
          delivery_tag: !phs payload.delivery_tag
      
      - return: !phs {
          message: `Task ${payload.task.id} completed by ${payload.worker_id}`,
          task: payload.task,
          processing_time: payload.task.complexity,
          completed_at: new Date().toISOString()
        }
```

## Pub/Sub Pattern

Implement publish-subscribe messaging:

```yaml title="pubsub.phlow"
name: Pub/Sub Pattern
version: 1.0.0
description: Publish-subscribe messaging pattern

main: cli
modules:
  - module: cli
    version: latest
    with:
      args:
        - name: mode
          description: Mode to run (publisher or subscriber)
          index: 1
          type: string
          required: true
          choices: ["publisher", "subscriber"]
        - name: topic
          description: Topic to publish/subscribe to
          index: 2
          type: string
          default: "notifications"
        - name: subscriber_name
          description: Subscriber name
          index: 3
          type: string
          default: "subscriber1"
  - module: amqp
    version: latest
    with:
      host: localhost
      port: 5672
      username: guest
      password: guest
      vhost: /

steps:
  - payload: !phs {
      mode: main.mode,
      topic: main.topic,
      subscriber_name: main.subscriber_name
    }
  
  # Publisher mode
  - assert: !phs payload.mode == "publisher"
    then:
      - log:
          message: !phs `Publishing to topic: ${payload.topic}`
      
      # Declare exchange
      - amqp:
          action: declare_exchange
          exchange: !phs payload.topic
          type: "fanout"
      
      # Create messages
      - payload: !phs {
          ...payload,
          messages: [
            { type: "user_registered", user_id: 123, email: "user@example.com" },
            { type: "order_created", order_id: 456, total: 99.99 },
            { type: "system_alert", level: "warning", message: "High CPU usage detected" }
          ]
        }
      
      # Publish messages
      - amqp:
          action: publish
          exchange: !phs payload.topic
          routing_key: ""
          message: !phs JSON.stringify(payload.messages[0])
      
      - return: !phs {
          message: `Published ${payload.messages.length} messages to ${payload.topic}`,
          messages: payload.messages
        }
  
  # Subscriber mode
  - assert: !phs payload.mode == "subscriber"
    then:
      - log:
          message: !phs `Subscribing to topic: ${payload.topic} as ${payload.subscriber_name}`
      
      # Declare exchange
      - amqp:
          action: declare_exchange
          exchange: !phs payload.topic
          type: "fanout"
      
      # Declare exclusive queue for this subscriber
      - amqp:
          action: declare_queue
          queue: !phs `${payload.topic}_${payload.subscriber_name}`
          exclusive: true
      
      # Bind queue to exchange
      - amqp:
          action: bind_queue
          queue: !phs `${payload.topic}_${payload.subscriber_name}`
          exchange: !phs payload.topic
          routing_key: ""
      
      # Consume messages
      - amqp:
          action: consume
          queue: !phs `${payload.topic}_${payload.subscriber_name}`
          auto_ack: true
      
      # Process message
      - payload: !phs {
          ...payload,
          received_message: JSON.parse(payload.message),
          received_at: new Date().toISOString()
        }
      
      - log:
          message: !phs `${payload.subscriber_name} received: ${payload.received_message.type}`
      
      - return: !phs {
          subscriber: payload.subscriber_name,
          message: payload.received_message,
          received_at: payload.received_at
        }
```

## Testing Messaging and RPC

Test messaging and RPC functionality:

```yaml title="messaging-tests.phlow"
name: Messaging Tests
version: 1.0.0
description: Test messaging and RPC functionality

modules:
  - module: rpc
    version: latest
  - module: amqp
    version: latest
    with:
      host: localhost
      port: 5672
      username: guest
      password: guest
      vhost: /

tests:
  # Test RPC add method
  - main:
      method: "add"
      params: { a: 5, b: 3 }
    payload: null
    assert: !phs payload.result == 8
  
  # Test RPC multiply method
  - main:
      method: "multiply"
      params: { a: 4, b: 6 }
    payload: null
    assert: !phs payload.result == 24
  
  # Test message queue
  - main:
      queue: "test_queue"
      message: "Hello Test"
    payload: null
    assert: !phs payload.sent == true

steps:
  # Test RPC functionality
  - assert: !phs main.method && main.params
    then:
      - payload: !phs {
          method: main.method,
          params: main.params,
          request_id: Math.random().toString(36).substr(2, 9)
        }
      
      # Handle add method
      - assert: !phs payload.method == "add"
        then:
          return: !phs {
            jsonrpc: "2.0",
            result: payload.params.a + payload.params.b,
            id: payload.request_id
          }
      
      # Handle multiply method
      - assert: !phs payload.method == "multiply"
        then:
          return: !phs {
            jsonrpc: "2.0",
            result: payload.params.a * payload.params.b,
            id: payload.request_id
          }
  
  # Test AMQP functionality
  - assert: !phs main.queue && main.message
    then:
      - amqp:
          action: declare_queue
          queue: !phs main.queue
          durable: false
      
      - amqp:
          action: publish
          exchange: ""
          routing_key: !phs main.queue
          message: !phs main.message
      
      - return: !phs { sent: true, queue: main.queue, message: main.message }
```

## Key Features Demonstrated

1. **AMQP Messaging**: Publish and consume messages with RabbitMQ
2. **RPC Communication**: Remote procedure calls with JSON-RPC
3. **Work Queue Pattern**: Distribute tasks across multiple workers
4. **Pub/Sub Pattern**: Publish-subscribe messaging for event-driven systems
5. **Message Acknowledgment**: Reliable message processing
6. **Queue Management**: Declare queues, exchanges, and bindings
7. **Error Handling**: Proper error handling for distributed systems
8. **Testing**: Comprehensive testing of messaging functionality

These examples show how to build distributed systems using Phlow's messaging and RPC capabilities.
