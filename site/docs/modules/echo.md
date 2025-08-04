---
sidebar_position: 3
title: Echo Module
hide_title: true
---

# Echo Module

O módulo Echo é um módulo simples e fundamental que retorna exatamente o que recebe como entrada. É útil para debug, testes, passagem de dados e como exemplo básico de implementação de módulos Phlow.

## 🚀 Funcionalidades

### Características Principais

- ✅ **Simplicidade**: Retorna exatamente o que recebe
- ✅ **Qualquer tipo**: Aceita qualquer tipo de entrada
- ✅ **Preservação de dados**: Mantém estrutura e tipo original
- ✅ **Performance**: Operação de passagem direta, sem processamento
- ✅ **Debug**: Útil para verificar dados em pipelines
- ✅ **Observabilidade**: Totalmente integrado com OpenTelemetry

## 📋 Configuração

### Configuração Básica

```phlow
steps:
  - name: "echo_step"
    use: "echo_module"
    input: "Hello, World!"
```

### Configuração com Dados Estruturados

```phlow
steps:
  - name: "echo_object"
    use: "echo_module"
    input:
      message: "Hello"
      timestamp: "2024-01-01T00:00:00Z"
      data:
        items: [1, 2, 3]
        active: true
```

## 🔧 Parâmetros

### Entrada (Input)
- **Tipo**: `any` (qualquer tipo)
- **Obrigatório**: `true`
- **Descrição**: A mensagem ou dados a serem ecoados
- **Padrão**: `null`

### Saída (Output)
- **Tipo**: `any` (mesmo tipo da entrada)
- **Obrigatório**: `true`
- **Descrição**: Os dados ecoados (idênticos à entrada)
- **Padrão**: `null`

## 💻 Exemplos de Uso

### Echo de String Simples

```phlow
steps:
  - name: "simple_echo"
    use: "echo_module"
    input: "Esta mensagem será ecoada"
    
  # Saída: "Esta mensagem será ecoada"
```

### Echo de Número

```phlow
steps:
  - name: "number_echo"
    use: "echo_module"
    input: 42
    
  # Saída: 42
```

### Echo de Boolean

```phlow
steps:
  - name: "boolean_echo"
    use: "echo_module"
    input: true
    
  # Saída: true
```

### Echo de Array

```phlow
steps:
  - name: "array_echo"
    use: "echo_module"
    input: [1, 2, 3, "teste", true]
    
  # Saída: [1, 2, 3, "teste", true]
```

### Echo de Objeto Complexo

```phlow
steps:
  - name: "object_echo"
    use: "echo_module"
    input:
      user:
        id: 123
        name: "João Silva"
        email: "joao@example.com"
        active: true
        preferences:
          theme: "dark"
          notifications: true
        tags: ["admin", "premium"]
      metadata:
        created_at: "2024-01-01T00:00:00Z"
        updated_at: "2024-01-15T14:30:00Z"
        version: "1.2.3"
    
  # Saída: (objeto idêntico ao input)
```

### Echo com Dados Dinâmicos

```phlow
steps:
  - name: "process_user"
    # Algum processamento que retorna dados do usuário
    
  - name: "echo_user_data"
    use: "echo_module"
    input: "{{ $process_user }}"
    
  # Saída: (dados do usuário do step anterior)
```

## 🔍 Casos de Uso

### 1. Debug de Pipeline

```phlow
steps:
  - name: "fetch_data"
    use: "http_request"
    input:
      url: "https://api.example.com/users"
      
  - name: "debug_response"
    use: "echo_module"
    input: "{{ $fetch_data }}"
    # Útil para ver exatamente o que a API retornou
    
  - name: "process_data"
    # Continua processamento...
```

### 2. Passagem de Dados

```phlow
steps:
  - name: "calculate_result"
    script: |
      let result = input.a + input.b;
      result * 2;
    
  - name: "pass_result"
    use: "echo_module"
    input: "{{ $calculate_result }}"
    
  - name: "format_output"
    input: "Resultado: {{ $pass_result }}"
```

### 3. Validação de Estruturas

```phlow
steps:
  - name: "create_user_object"
    script: |
      {
        id: 123,
        name: "Usuário Teste",
        email: "test@example.com",
        created_at: new Date().toISOString()
      }
    
  - name: "validate_structure"
    use: "echo_module"
    input: "{{ $create_user_object }}"
    # Verifica se o objeto foi criado corretamente
    
  - name: "save_user"
    use: "database_save"
    input: "{{ $validate_structure }}"
```

### 4. Testes e Desenvolvimento

```phlow
steps:
  - name: "mock_api_response"
    use: "echo_module"
    input:
      status: "success"
      data:
        users: [
          { id: 1, name: "Alice" },
          { id: 2, name: "Bob" }
        ]
      timestamp: "2024-01-01T00:00:00Z"
    
  - name: "process_users"
    # Processa como se viesse de uma API real
    input: "{{ $mock_api_response.data.users }}"
```

## 🌐 Exemplo Completo

```phlow
name: "echo-demo"
version: "1.0.0"
description: "Demonstração do módulo Echo"

modules:
  - name: "echo_module"
    module: "echo"
    version: "0.0.1"

steps:
  - name: "echo_string"
    use: "echo_module"
    input: "Hello from Echo!"
    
  - name: "echo_number"
    use: "echo_module"
    input: 3.14159
    
  - name: "echo_complex_object"
    use: "echo_module"
    input:
      application:
        name: "MyApp"
        version: "2.1.0"
        config:
          debug: true
          max_connections: 100
          features: ["auth", "cache", "logging"]
      environment:
        stage: "production"
        region: "us-east-1"
        
  - name: "echo_with_interpolation"
    use: "echo_module"
    input: "App: {{ $echo_complex_object.application.name }} v{{ $echo_complex_object.application.version }}"
    
  - name: "final_output"
    script: |
      {
        string_echo: $echo_string,
        number_echo: $echo_number,
        object_echo: $echo_complex_object,
        interpolated_echo: $echo_with_interpolation
      }
```

## 📊 Observabilidade

O módulo Echo herda a observabilidade padrão do Phlow SDK:

- **Tracing**: Cada execução gera spans OpenTelemetry
- **Logging**: Logs estruturados para debug
- **Metrics**: Métricas de performance e uso
- **Context**: Propagação de contexto entre steps

## 🔒 Segurança

- **Preservação de dados**: Não modifica nem expõe dados sensíveis
- **Sem efeitos colaterais**: Operação puramente funcional
- **Memória**: Passa referências quando possível para eficiência

## 📈 Performance

- **Latência mínima**: Operação de passagem direta
- **Memória eficiente**: Sem cópias desnecessárias
- **Threading**: Suporte completo a execução assíncrona
- **Escalabilidade**: Sem limitações de throughput

## 🛠️ Implementação

O módulo Echo é implementado de forma minimalista:

```rust
pub async fn echo(rx: ModuleReceiver) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    listen!(rx, move |package: ModulePackage| async {
        let input = package.input().unwrap_or(Value::Null);
        sender_safe!(package.sender, input.into());
    });
    
    Ok(())
}
```

## 🏷️ Tags

- echo
- debug
- passthrough
- testing
- utility

---

**Versão**: 0.0.1  
**Autor**: Philippe Assis `<codephilippe@gmail.com>`
**Licença**: MIT  
**Repositório**: https://github.com/phlowdotdev/phlow
