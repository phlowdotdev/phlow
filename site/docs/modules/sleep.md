---
sidebar_position: 9
title: Sleep Module
hide_title: true
---

# Sleep Module

The Sleep module provides functionality to pause execution for a specific period, useful for delays, throttling, and synchronization in workflows.

## 🚀 Features

### Key Features

- ✅ **Multiple time units**: milliseconds, seconds, minutes, hours
- ✅ **Flexibility**: Accepts any time unit
- ✅ **Performance**: Does not block other flows
- ✅ **Observability**: Integrated logging
- ✅ **Simplicity**: Intuitive interface

## 📝 Configuration

### Basic Configuration

```phlow
steps:
  - name: "wait_5_seconds"
    use: "sleep_module"
    input:
      seconds: 5
```

## 🔧 Parameters

### Input
- `milliseconds` (integer, optional): Time in milliseconds (default: 0)
- `seconds` (integer, optional): Time in seconds (default: 0)
- `minutes` (integer, optional): Time in minutes (default: 0)
- `hours` (integer, optional): Time in hours (default: 0)

### Output
- Returns `null` after sleep period

## 💻 Exemplos de Uso

### Sleep em Diferentes Unidades

```phlow
steps:
  - name: "quick_pause"
    use: "sleep_module"
    input:
      milliseconds: 500
      
  - name: "short_delay"
    use: "sleep_module"
    input:
      seconds: 10
      
  - name: "medium_wait"
    use: "sleep_module"
    input:
      minutes: 2
      
  - name: "long_delay"
    use: "sleep_module"
    input:
      hours: 1
```

### Throttling de API

```phlow
steps:
  - name: "api_call_1"
    use: "http_client"
    input:
      method: "GET"
      url: "https://api.example.com/data/1"
      
  - name: "throttle_delay"
    use: "sleep_module"
    input:
      milliseconds: 100
      
  - name: "api_call_2"
    use: "http_client"
    input:
      method: "GET"
      url: "https://api.example.com/data/2"
```

### Retry com Backoff

```phlow
steps:
  - name: "attempt_operation"
    use: "some_module"
    input:
      operation: "risky_operation"
      
  - name: "check_success"
    condition:
      left: "{{ $attempt_operation.success }}"
      operator: "equals"
      right: false
    then:
      # Esperar antes de retry
      use: "sleep_module"
      input:
        seconds: 5
    else:
      return: "{{ $attempt_operation.result }}"
```

## 🌐 Exemplo Completo

```phlow
name: "batch-processor"
version: "1.0.0"
description: "Processamento em lote com delays"

modules:
  - name: "sleep_module"
    module: "sleep"
    version: "0.0.1"
    
  - name: "http_client"
    module: "http_request"
    
  - name: "logger"
    module: "log"

steps:
  - name: "start_processing"
    use: "logger"
    input:
      message: "Iniciando processamento em lote"
      
  - name: "process_batch_1"
    use: "http_client"
    input:
      method: "POST"
      url: "https://api.example.com/process"
      body: '{"batch": 1, "items": [1, 2, 3]}'
      
  - name: "log_batch_1"
    use: "logger"
    input:
      message: "Lote 1 processado, aguardando..."
      
  - name: "wait_between_batches"
    use: "sleep_module"
    input:
      seconds: 30
      
  - name: "process_batch_2"
    use: "http_client"
    input:
      method: "POST"
      url: "https://api.example.com/process"
      body: '{"batch": 2, "items": [4, 5, 6]}'
      
  - name: "log_batch_2"
    use: "logger"
    input:
      message: "Lote 2 processado, aguardando..."
      
  - name: "final_wait"
    use: "sleep_module"
    input:
      minutes: 5
      
  - name: "finalize_processing"
    use: "http_client"
    input:
      method: "POST"
      url: "https://api.example.com/finalize"
      body: '{"status": "completed"}'
      
  - name: "completion_log"
    use: "logger"
    input:
      message: "Processamento completo!"
```

### Rate Limiting Example

```phlow
name: "api-rate-limiter"
version: "1.0.0"

modules:
  - name: "sleep_module"
    module: "sleep"
    
  - name: "api_client"
    module: "http_request"

steps:
  - name: "api_requests"
    loop:
      items: ["item1", "item2", "item3", "item4", "item5"]
      steps:
        - name: "make_request"
          use: "api_client"
          input:
            method: "GET"
            url: "https://api.example.com/data/{{ $loop.item }}"
            
        - name: "check_rate_limit"
          condition:
            left: "{{ $make_request.response.status_code }}"
            operator: "equals"
            right: 429
          then:
            # Rate limit hit, wait longer
            use: "sleep_module"
            input:
              seconds: 60
          else:
            # Normal delay between requests
            use: "sleep_module"
            input:
              milliseconds: 200
```

## 📊 Casos de Uso Comuns

### 1. Debouncing
```phlow
steps:
  - name: "user_input"
    # Captura entrada do usuário
    
  - name: "debounce_delay"
    use: "sleep_module"
    input:
      milliseconds: 300
      
  - name: "process_input"
    # Processa apenas se não houver nova entrada
```

### 2. Scheduling
```phlow
steps:
  - name: "schedule_check"
    script: |
      let now = new Date();
      let targetTime = new Date(now.getTime() + 3600000); // 1 hora
      targetTime
      
  - name: "wait_until_scheduled"
    use: "sleep_module"
    input:
      hours: 1
      
  - name: "execute_scheduled_task"
    # Executa tarefa agendada
```

### 3. Circuit Breaker
```phlow
steps:
  - name: "service_call"
    use: "external_service"
    
  - name: "handle_failure"
    condition:
      left: "{{ $service_call.success }}"
      operator: "equals"
      right: false
    then:
      # Circuit breaker - wait before next attempt
      use: "sleep_module"
      input:
        minutes: 5
```

## 🔧 Configuração Avançada

### Combinação de Unidades
```phlow
# Note: Apenas a primeira unidade encontrada é usada
steps:
  - name: "complex_sleep"
    use: "sleep_module"
    input:
      # Apenas seconds será usado (primeiro encontrado)
      seconds: 30
      minutes: 1  # Ignorado
      hours: 1    # Ignorado
```

### Sleep Dinâmico
```phlow
steps:
  - name: "calculate_delay"
    script: |
      let delay = Math.min(Math.max($attempt_count * 1000, 100), 10000);
      delay
      
  - name: "dynamic_sleep"
    use: "sleep_module"
    input:
      milliseconds: "{{ $calculate_delay }}"
```

## 🏷️ Tags

- sleep
- wait
- delay
- throttling
- timing

---

**Versão**: 0.0.1  
**Autor**: Philippe Assis `<codephilippe@gmail.com>`  
**Licença**: MIT  
**Repositório**: https://github.com/phlowdotdev/phlow
