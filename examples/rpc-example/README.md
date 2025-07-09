# RPC Module Example

Este exemplo demonstra como usar o módulo RPC do Phlow para comunicação cliente-servidor usando tarpc.

## Funcionalidades

- **Servidor RPC**: Quando usado como módulo principal, inicia um servidor RPC
- **Cliente RPC**: Quando usado em steps, funciona como cliente para fazer chamadas RPC
- **Health Check**: Endpoint para verificar se o servidor está funcionando
- **Service Info**: Obter informações sobre o serviço
- **Configuração flexível**: Timeout, max connections, host e porta configuráveis

## Configuração

### Servidor RPC

```yaml
main: rpc_server
modules:
  - module: rpc
    version: latest
    name: rpc_server
    with:
      host: "127.0.0.1"         # Host do servidor
      port: 8090                # Porta do servidor
      timeout_ms: 10000         # Timeout em ms
      max_connections: 100      # Máximo de conexões simultâneas
      service_name: "my-service" # Nome do serviço
```

### Cliente RPC

```yaml
modules:
  - module: rpc
    version: latest
    name: rpc_client
    with:
      host: "127.0.0.1"         # Host do servidor RPC
      port: 8090                # Porta do servidor RPC
      timeout_ms: 5000          # Timeout para chamadas
      service_name: "my-service" # Nome do serviço
```

## Uso

### 1. Iniciando o Servidor

```bash
phlow run server.yaml
```

### 2. Usando o Cliente

```bash
phlow run client.yaml
```

### 3. Testando com curl

#### Chamada RPC normal:
```bash
curl -X POST http://localhost:8080/ \
  -H "Content-Type: application/json" \
  -d '{"data": "test", "value": 123}'
```

#### Health Check:
```bash
curl -X GET http://localhost:8080/
```

## Tipos de Chamadas

### Chamada RPC Padrão

```yaml
- use: rpc_client
  input:
    method: "my_method"
    params:
      key: "value"
      number: 42
    headers:
      "Content-Type": "application/json"
```

### Health Check

```yaml
- use: rpc_client
  input:
    action: "health"
```

### Service Info

```yaml
- use: rpc_client
  input:
    action: "info"
```

## Estrutura da Resposta

### Resposta RPC Normal
```json
{
  "result": {...},
  "error": null,
  "headers": {...}
}
```

### Resposta Health Check
```json
{
  "healthy": true,
  "service": "my-service",
  "address": "127.0.0.1:8090"
}
```

### Resposta Service Info
```json
{
  "service_name": "my-service",
  "version": "0.1.0",
  "status": "running",
  "hostname": "localhost"
}
```

## Configurações Opcionais

- `host`: Endereço IP do servidor (padrão: "127.0.0.1")
- `port`: Porta do servidor (padrão: 8080)
- `timeout_ms`: Timeout em milissegundos (padrão: 5000)
- `max_connections`: Máximo de conexões simultâneas (padrão: 100)
- `service_name`: Nome do serviço (padrão: "default")

## Logs

O módulo RPC produz logs detalhados para debugging:

- Conexões estabelecidas
- Chamadas RPC recebidas/enviadas
- Erros de conexão
- Timeouts
- Respostas processadas
