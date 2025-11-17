# AMQP Module

The AMQP module provides a comprehensive interface for sending and receiving messages using the AMQP 0.9.1 protocol (Advanced Message Queuing Protocol), compatible with RabbitMQ.

## 🚀 Features

### Operation Modes

- **Consumer Mode**: When configured as 'main', consumes messages from a specified queue
- **Producer Mode**: When used with 'use' in steps, sends messages to exchanges or queues

### Main Features

- ✅ Auto-declaration of exchanges, queues, and bindings
- ✅ Configuration of durability, exclusivity, and auto-deletion
- ✅ Support for all AMQP exchange types (direct, fanout, topic, headers)
- ✅ Support for custom message headers
- ✅ Import and automatic creation of vhosts, exchanges, queues, and bindings
- ✅ Automatic binding of queues to exchanges
- ✅ SSL/TLS support via OpenSSL
- ✅ Full observability with OpenTelemetry tracing
- ✅ Import RabbitMQ definitions via Management API
- ✅ Automatic reconnection in case of channel failure
- ✅ Dead Letter Queue (DLQ) support with configurable retry attempts
- ✅ Error handling with retry mechanisms

## 📋 Configuration

### Basic Configuration

```yaml
modules:
  - name: "message_processor"
    module: "amqp"
    version: "0.0.3"
    with:
      host: "localhost"
      port: 5672
      username: "guest"
      password: "guest"
      vhost: "/"
      routing_key: "process.task"
      exchange: "task_exchange"
      exchange_type: "direct"
      queue_name: "task_queue"
      consumer_tag: "consumer_1"
```

### Configuration with URI

```yaml
modules:
  - name: "message_processor"
    module: "amqp"
    with:
      uri: "amqp://user:password@rabbitmq.example.com:5672/production"
      routing_key: "process.task"
      exchange: "task_exchange"
```

### Configuration with RabbitMQ Definitions

```yaml
modules:
  - name: "message_processor"
    module: "amqp"
    with:
      host: "localhost"
      routing_key: "process.task"
      definition:
        vhosts:
          - name: "/"
        exchanges:
          - name: "task_exchange"
            type: "direct"
            durable: true
            auto_delete: false
            vhost: "/"
        queues:
          - name: "task_queue"
            durable: true
            auto_delete: false
            vhost: "/"
        bindings:
          - source: "task_exchange"
            destination: "task_queue"
            routing_key: "process.task"
            vhost: "/"
```

## 🔧 Configuration Parameters

### Connection
- `uri` (string, optional): Full AMQP connection URI
- `host` (string, optional): AMQP server host (default: "localhost")
- `port` (integer, optional): AMQP server port (default: 5672)
- `username` (string, optional): Username (default: "guest")
- `password` (string, optional): Password (default: "guest")
- `vhost` (string, optional): Virtual host (default: "/")

### Routing
- `routing_key` (string, required): AMQP routing key
- `exchange` (string, optional): Exchange name
- `exchange_type` (enum, optional): Exchange type [direct, fanout, topic, headers]
- `queue_name` (string, optional): Queue name (uses routing_key if not specified)
- `consumer_tag` (string, optional): Consumer tag

### Management
- `management_port` (integer, optional): Management API port (default: 15672)
- `definition` (object, optional): RabbitMQ definitions for automatic import

### Error Handling
- `max_retry` (integer, optional): Maximum number of retry attempts before sending to DLQ (default: 3)
- `dlq_enable` (boolean, optional): Enable Dead Letter Queue functionality (default: true)

### Concurrency
- `max_concurrency` (integer, optional): Limite máximo de mensagens processadas simultaneamente pelo consumidor. Implementado via AMQP QoS `prefetch_count`. Use `0` para ilimitado (padrão: 0).

## 📨 Usage as Consumer (Main Module)

```yaml
name: "message-consumer"
main: "amqp_consumer"

modules:
  - name: "amqp_consumer"
    module: "amqp"
    with:
      host: "rabbitmq.example.com"
      queue_name: "input_queue"
      routing_key: "process.task"
      exchange: "task_exchange"

steps:
  - name: "process_message"
    # Processes the received message
    # The message is available in `$input`
```

## 📤 Usage as Producer (in Steps)

```yaml
steps:
  - name: "send_message"
    use: "amqp_producer"
    input:
      message: '{"task": "process_data", "id": 123}'
      headers:
        content-type: "application/json"
        priority: "high"
        timestamp: "2024-01-01T00:00:00Z"
        correlation-id: "abc-123"
```

## 🔄 Exchange Types

### Direct Exchange
```yaml
with:
  exchange: "direct_exchange"
  exchange_type: "direct"
  routing_key: "exact.match"
```

### Fanout Exchange
```yaml
with:
  exchange: "fanout_exchange"
  exchange_type: "fanout"
  # routing_key is not required for fanout
```

### Topic Exchange
```yaml
with:
  exchange: "topic_exchange"
  exchange_type: "topic"
  routing_key: "orders.*.priority"
```

### Headers Exchange
```yaml
with:
  exchange: "headers_exchange"
  exchange_type: "headers"
  # routing_key is not required for headers
```

## 📊 Observability

The module automatically generates OpenTelemetry traces with the following attributes:

### Span Attributes
- `messaging.system`: "rabbitmq"
- `messaging.destination.name`: queue name
- `messaging.destination.kind`: "queue"
- `messaging.operation`: "receive" or "publish"
- `messaging.protocol`: "AMQP"
- `messaging.protocol_version`: "0.9.1"
- `messaging.rabbitmq.consumer_tag`: consumer tag
- `messaging.client.id`: client hostname
- `messaging.message.payload_size_bytes`: message size
- `messaging.message.conversation_id`: conversation ID

## 🛠️ Definitions Import

The module supports automatic import of RabbitMQ definitions via Management API:

```yaml
with:
  definition:
    vhosts:
      - name: "production"
    exchanges:
      - name: "orders"
        type: "topic"
        durable: true
        vhost: "production"
    queues:
      - name: "order_processing"
        durable: true
        vhost: "production"
    bindings:
      - source: "orders"
        destination: "order_processing"
        routing_key: "order.created"
        vhost: "production"
```

## 🔍 Producer Response

```json
{
  "success": true,
  "error_message": null
}
```

In case of error:
```json
{
  "success": false,
  "error_message": "Error description"
}
```

## 🌐 Complete Example

```yaml
name: "order-processing-system"
version: "1.0.0"
main: "order_consumer"

modules:
  - name: "order_consumer"
    module: "amqp"
    with:
      host: "rabbitmq.company.com"
      port: 5672
      username: "app_user"
      password: "secure_password"
      vhost: "production"
      exchange: "orders"
      exchange_type: "topic"
      routing_key: "order.created"
      queue_name: "order_processing"
      consumer_tag: "order_processor_1"
      definition:
        exchanges:
          - name: "orders"
            type: "topic"
            durable: true
            vhost: "production"
        queues:
          - name: "order_processing"
            durable: true
            vhost: "production"
        bindings:
          - source: "orders"
            destination: "order_processing"
            routing_key: "order.created"
            vhost: "production"

  - name: "notification_sender"
    module: "amqp"
    with:
      host: "rabbitmq.company.com"
      exchange: "notifications"
      routing_key: "notification.email"

steps:
  - name: "process_order"
    # Processes the received order
    
  - name: "send_notification"
    use: "notification_sender"
    input:
      message: '{"type": "order_processed", "order_id": "{{ $input.order_id }}"}'
      headers:
        content-type: "application/json"
        priority: "normal"
```

## 🔒 Security

- Full SSL/TLS support via OpenSSL
- Authentication with username and password
- Support for virtual hosts for isolation
- Secure credential configuration via environment variables

## 📈 Performance

- Automatic reconnection in case of failure
- Asynchronous message processing
- Efficient connection resource management
- Support for publish confirmations

## 🏷️ Tags

- queue
- message
- rabbitmq
- producer
- consumer
- amqp
- messaging

---

**Version**: 0.0.3  
**Author**: Philippe Assis <codephilippe@gmail.com>  
**License**: MIT  
**Repository**: https://github.com/phlowdotdev/phlow

