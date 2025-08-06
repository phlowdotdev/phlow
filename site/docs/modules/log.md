---
sidebar_position: 6
title: Log Module
hide_title: true
---

# Log Module

The Log module provides structured logging functionality for Phlow applications, allowing you to record messages with different severity levels.

## 🚀 Features

### Key Features

- ✅ **Multiple log levels**: info, debug, warn, error
- ✅ **Structured logging**: Compatible with env_logger
- ✅ **Flexible configuration**: Via PHLOW_LOG environment variable
- ✅ **Observability**: Integration with OpenTelemetry
- ✅ **Performance**: Asynchronous logging without blocking

## 📋 Configuração

### Configuração Básica

```phlow
steps:
  - name: "log_info"
    use: "logger"
    input:
      level: "info"
      message: "Aplicação iniciada com sucesso"
```

### Configuração com Variáveis de Ambiente

```bash
# Nível de log padrão
export PHLOW_LOG="debug"  # info, debug, warn, error
```

## 🔧 Parâmetros

### Entrada (input)
- `level` (string, opcional): Nível do log [info, debug, warn, error] (padrão: "info")
- `message` (string, obrigatório): Mensagem a ser registrada

### Saída (output)
- Retorna `null` após processar o log

## 💻 Exemplos de Uso

### Logs de Diferentes Níveis

```phlow
steps:
  - name: "log_info"
    use: "logger"
    input:
      level: "info"
      message: "Processamento iniciado"
      
  - name: "log_debug"
    use: "logger"
    input:
      level: "debug"
      message: "Variável x = {{ $x }}"
      
  - name: "log_warn"
    use: "logger"
    input:
      level: "warn"
      message: "Configuração não encontrada, usando padrão"
      
  - name: "log_error"
    use: "logger"
    input:
      level: "error"
      message: "Falha na conexão com banco de dados"
```

### Logging em Pipeline

```phlow
steps:
  - name: "start_log"
    use: "logger"
    input:
      message: "Iniciando processamento do usuário {{ $user_id }}"
      
  - name: "process_user"
    script: |
      // Processamento do usuário
      let result = { id: $user_id, status: "processed" };
      result
      
  - name: "success_log"
    use: "logger"
    input:
      level: "info"
      message: "Usuário {{ $process_user.id }} processado com sucesso"
      
  - name: "debug_log"
    use: "logger"
    input:
      level: "debug"
      message: "Dados do usuário: {{ $process_user }}"
```

## 🌐 Exemplo Completo

```phlow
name: "logging-example"
version: "1.0.0"
description: "Exemplo de uso do módulo Log"

modules:
  - name: "logger"
    module: "log"
    version: "0.0.1"

steps:
  - name: "start_application"
    use: "logger"
    input:
      level: "info"
      message: "Aplicação iniciada em {{ new Date().toISOString() }}"
      
  - name: "load_config"
    script: |
      // Simular carregamento de configuração
      let config = {
        database: "postgresql://localhost:5432/mydb",
        port: 3000,
        debug: true
      };
      config
      
  - name: "log_config"
    use: "logger"
    input:
      level: "debug"
      message: "Configuração carregada: {{ JSON.stringify($load_config) }}"
      
  - name: "validate_config"
    condition:
      left: "{{ $load_config.database }}"
      operator: "exists"
      right: true
    then:
      use: "logger"
      input:
        level: "info"
        message: "Configuração de banco de dados válida"
    else:
      use: "logger"
      input:
        level: "error"
        message: "Configuração de banco de dados ausente"
        
  - name: "performance_warning"
    condition:
      left: "{{ $load_config.debug }}"
      operator: "equals"
      right: true
    then:
      use: "logger"
      input:
        level: "warn"
        message: "Modo debug ativado - performance pode ser afetada"
        
  - name: "final_log"
    use: "logger"
    input:
      level: "info"
      message: "Aplicação configurada e pronta para usar"
```

## 🔧 Configuração Avançada

### Níveis de Log

```bash
# Apenas erros
export PHLOW_LOG="error"

# Warnings e erros
export PHLOW_LOG="warn"

# Informações, warnings e erros
export PHLOW_LOG="info"

# Todos os logs incluindo debug
export PHLOW_LOG="debug"
```

### Formatação de Logs

O módulo usa env_logger, que pode ser configurado:

```bash
# Formato personalizado
export RUST_LOG_STYLE="always"
export PHLOW_LOG="debug"
```

## 📊 Saída de Exemplo

```
[2024-01-01T00:00:00Z INFO  phlow] Aplicação iniciada com sucesso
[2024-01-01T00:00:01Z DEBUG phlow] Variável x = 42
[2024-01-01T00:00:02Z WARN  phlow] Configuração não encontrada, usando padrão
[2024-01-01T00:00:03Z ERROR phlow] Falha na conexão com banco de dados
```

## 🏷️ Tags

- log
- echo
- print
- logging
- debug

---

**Versão**: 0.0.1  
**Autor**: Philippe Assis `<codephilippe@gmail.com>`  
**Licença**: MIT  
**Repositório**: https://github.com/phlowdotdev/phlow
