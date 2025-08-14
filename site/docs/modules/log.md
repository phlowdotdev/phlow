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

### Configuração Básica (Sintaxe Recomendada)

```phlow
steps:
  - use: log
    input:
      level: "info"
      message: "Aplicação iniciada com sucesso"
```

### Configuração Básica (Sintaxe Legada - Ainda Suportada)

```phlow
steps:
  - log:
      level: "info"
      message: "Aplicação iniciada com sucesso"
```

**Nota:** Ambas as sintaxes são suportadas. A sintaxe legada é automaticamente transformada para a nova sintaxe durante o processamento.

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

### Logs de Diferentes Níveis (Nova Sintaxe)

```phlow
steps:
  - use: log
    input:
      level: "info"
      message: "Processamento iniciado"
      
  - use: log
    input:
      level: "debug"
      message: !phs `Variável x = ${main.x}`
      
  - use: log
    input:
      level: "warn"
      message: "Configuração não encontrada, usando padrão"
      
  - use: log
    input:
      level: "error"
      message: "Falha na conexão com banco de dados"
```

### Logs de Diferentes Níveis (Sintaxe Legada - Transformada Automaticamente)

```phlow
steps:
  - log:
      level: "info"
      message: "Processamento iniciado"
      
  - log:
      level: "debug"
      message: !phs `Variável x = ${main.x}`
      
  - log:
      level: "warn"
      message: "Configuração não encontrada, usando padrão"
      
  - log:
      level: "error"
      message: "Falha na conexão com banco de dados"
```

### Logging com Blocos de Código

```phlow
steps:
  - payload: !phs {
      let user = main.user;
      let timestamp = new Date().toISOString();
      
      #{
        id: user.id,
        name: user.name,
        loginTime: timestamp,
        sessionId: Math.random().toString(36)
      }
    }
    
  - use: log
    input:
      level: "info"
      message: !phs {
        let session = payload;
        let status = session.id ? "success" : "failed";
        
        `User login ${status}: ${session.name} (ID: ${session.id}) at ${session.loginTime}`
      }
```

### Logging em Pipeline

```phlow
steps:
  - use: log
    input:
      message: !phs `Iniciando processamento do usuário ${main.user_id}`
      
  - payload: !phs {
      let userId = main.user_id;
      let processedAt = new Date().toISOString();
      
      #{
        id: userId,
        status: "processed",
        timestamp: processedAt,
        result: `User ${userId} processed successfully`
      }
    }
      
  - use: log
    input:
      level: "info"
      message: !phs `Usuário ${payload.id} processado com sucesso`
      
  - use: log
    input:
      level: "debug"
      message: !phs {
        let data = JSON.stringify(payload, null, 2);
        `Dados do usuário processado: ${data}`
      }
```

## 🌐 Exemplo Completo

```phlow
name: "logging-example"
version: "1.0.0"
description: "Exemplo de uso do módulo Log com novas funcionalidades"

modules:
  - module: log
    version: latest

steps:
  - use: log
    input:
      level: "info"
      message: !phs {
        let timestamp = new Date().toISOString();
        `Aplicação iniciada em ${timestamp}`
      }
      
  - payload: !phs {
      // Simular carregamento de configuração
      let config = {
        database: "postgresql://localhost:5432/mydb",
        port: 3000,
        debug: true,
        version: "1.0.0"
      };
      
      config
    }
      
  - use: log
    input:
      level: "debug"
      message: !phs {
        let configStr = JSON.stringify(payload, null, 2);
        `Configuração carregada: ${configStr}`
      }
      
  - assert: !phs payload.database != null
    then:
      - use: log
        input:
          level: "info"
          message: "Configuração de banco de dados válida"
    else:
      - use: log
        input:
          level: "error"
          message: "Configuração de banco de dados ausente"
        
  - assert: !phs payload.debug === true
    then:
      - use: log
        input:
          level: "warn"
          message: !phs {
            let version = payload.version;
            `Modo debug ativado na versão ${version} - performance pode ser afetada`
          }
        
  - use: log
    input:
      level: "info"
      message: !phs {
        let port = payload.port;
        let dbHost = payload.database.split("://")[1].split("/")[0];
        
        `Aplicação configurada - Porta: ${port}, DB: ${dbHost}`
      }
```

### Exemplo com Sintaxe Mista (Legada + Nova)

```phlow
modules:
  - module: log

steps:
  # Nova sintaxe
  - use: log
    input:
      message: "Iniciando com nova sintaxe"
      
  # Sintaxe legada (será transformada automaticamente)
  - log:
      level: "debug"
      message: "Esta é a sintaxe legada"
      
  # Nova sintaxe com bloco de código
  - use: log
    input:
      level: "info"
      message: !phs {
        let mode = "mixed";
        let timestamp = new Date().toISOString();
        
        `Modo ${mode} ativo em ${timestamp}`
      }
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
