---
sidebar_position: 11
title: Cache Module
hide_title: true
---

# Cache Module

The Cache module provides comprehensive in-memory caching functionality for Phlow applications, allowing temporary data storage with high performance, TTL (Time To Live) control, and advanced filtering and sorting operations.

## 🚀 Features

### Key Features

- ✅ **High Performance**: O(1) access for get/set operations with QuickLeaf technology
- ✅ **Automatic TTL**: Automatic expiration of items with configurable Time To Live
- ✅ **LRU Eviction**: Automatic removal of least recently used items when capacity is reached
- ✅ **Advanced Filtering**: Filters by prefix, suffix, and custom patterns
- ✅ **Sorting and Pagination**: Ordered listing with limit/offset support
- ✅ **Real-time Statistics**: Hit rate, memory usage, and operation counters
- ✅ **Thread Safety**: Safe for concurrent access across multiple Phlow flows
- ✅ **Action-based API**: Multiple operations through a unified interface
- ✅ **Manual Cleanup**: Manual cleanup of expired items when needed

## 📋 Configuração

### Configuração Básica

```phlow
modules:
  - module: cache
    with:
      capacity: 1000      # Máximo de 1000 itens
      default_ttl: 3600   # TTL padrão de 1 hora
```

### Configuração para Produção

```phlow
modules:
  - module: cache
    with:
      capacity: 10000     # Alta capacidade para produção
      default_ttl: 1800   # 30 minutos padrão
```

### Configuração para Desenvolvimento/Teste

```phlow
modules:
  - module: cache
    with:
      capacity: 100       # Capacidade pequena para testes
      default_ttl: 300    # 5 minutos para desenvolvimento
```

## 🔧 Parâmetros de Configuração

### Configuração do Módulo (with)
- `capacity` (integer, opcional): Número máximo de itens no cache (padrão: 1000)
- `default_ttl` (integer, opcional): TTL padrão em segundos para novos itens

### Entrada (input)
- `action` (string, obrigatório): Ação a executar ["set", "get", "remove", "clear", "exists", "list", "cleanup", "stats"]
- `key` (string): Chave do item (obrigatório para set, get, remove, exists)
- `value` (any): Valor a armazenar (obrigatório para set)
- `ttl` (integer, opcional): TTL em segundos para o item específico
- `filter_type` (string, opcional): Tipo de filtro para list ["prefix", "suffix", "pattern"]
- `filter_prefix` (string, opcional): Prefixo para filtrar (usado com list)
- `filter_suffix` (string, opcional): Sufixo para filtrar (usado com list)  
- `order` (string, opcional): Ordenação para list ["asc", "desc"] (padrão: "asc")
- `limit` (integer, opcional): Número máximo de itens para list
- `offset` (integer, opcional): Número de itens para pular em list (padrão: 0)

### Saída (output)
- `success` (boolean): Se a operação foi bem-sucedida
- `error` (string): Mensagem de erro (se falhou)
- `found` (boolean): Se o item foi encontrado (get, exists)
- `value` (any): Valor recuperado (get)
- `cached` (boolean): Se o item foi armazenado (set)
- `removed` (boolean): Se o item foi removido (remove)
- `previous_size` (integer): Tamanho anterior do cache (clear)
- `items` (array): Lista de itens (list)
- `total_count` (integer): Total de itens encontrados (list)
- `has_more` (boolean): Se há mais itens disponíveis (list)
- `cleaned_count` (integer): Número de itens limpos (cleanup)
- `stats` (object): Estatísticas detalhadas do cache (stats)

## 💻 Exemplos de Uso

### Operações Básicas de Cache

#### Armazenar Dados (Set)

```phlow
steps:
  - use: cache
    input:
      action: set
      key: "user:123"
      value:
        id: 123
        name: "João Silva"
        email: "joao@example.com"
        role: "admin"
      ttl: 3600  # Expira em 1 hora
```

#### Recuperar Dados (Get)

```phlow
steps:
  - use: cache
    input:
      action: get
      key: "user:123"
  
  - assert: !phs payload.found
    then:
      - return: !phs payload.value
    else:
      - return: 
          error: "Usuário não encontrado no cache"
```

#### Verificar Existência (Exists)

```phlow
steps:
  - use: cache
    input:
      action: exists
      key: "user:123"
  
  - return: !phs `Usuário existe no cache: ${payload.found}`
```

#### Remover Item (Remove)

```phlow
steps:
  - use: cache
    input:
      action: remove
      key: "user:123"
  
  - assert: !phs payload.removed
    then:
      - return: "Usuário removido com sucesso"
    else:
      - return: "Usuário não estava no cache"
```

#### Limpar Cache Completo (Clear)

```phlow
steps:
  - use: cache
    input:
      action: clear
  
  - return: !phs `Cache limpo, ${payload.previous_size} itens removidos`
```

### Operações Avançadas

#### Listagem com Filtros

##### Filtrar por Prefixo
```phlow
steps:
  - use: cache
    input:
      action: list
      filter_type: "prefix"
      filter_prefix: "user:"
      order: "asc"
      limit: 10
```

##### Filtrar por Sufixo
```phlow
steps:
  - use: cache
    input:
      action: list
      filter_type: "suffix"
      filter_suffix: ":session"
      order: "desc"
      limit: 20
```

##### Filtrar por Padrão (Prefixo + Sufixo)
```phlow
steps:
  - use: cache
    input:
      action: list
      filter_type: "pattern"
      filter_prefix: "cache_"
      filter_suffix: "_data"
      limit: 50
```

#### Paginação

```phlow
steps:
  # Primeira página
  - use: cache
    input:
      action: list
      order: "asc"
      limit: 10
      offset: 0
  
  # Segunda página
  - use: cache
    input:
      action: list
      order: "asc"
      limit: 10
      offset: 10
```

#### Limpeza Manual (Cleanup)

```phlow
steps:
  - use: cache
    input:
      action: cleanup
  
  - return: !phs `${payload.cleaned_count} itens expirados removidos`
```

#### Estatísticas do Cache (Stats)

```phlow
steps:
  - use: cache
    input:
      action: stats
  
  - return: !phs payload.stats
```

## 📊 Tipos de Dados Suportados

### Strings
```phlow
- use: cache
  input:
    action: set
    key: "message"
    value: "Olá, mundo!"
    ttl: 300
```

### Números
```phlow
- use: cache
  input:
    action: set
    key: "counter"
    value: 42
    ttl: 600
```

### Objetos Complexos
```phlow
- use: cache
  input:
    action: set
    key: "user:profile"
    value:
      id: 123
      name: "Ana Costa"
      preferences:
        theme: "dark"
        language: "pt-BR"
      settings:
        notifications: true
        privacy: "public"
    ttl: 1800
```

### Arrays
```phlow
- use: cache
  input:
    action: set
    key: "user:permissions"
    value: ["read", "write", "admin", "delete"]
    ttl: 3600
```

## 🌐 Exemplos Completos

### Sistema de Sessões de Usuário

```phlow
name: "user-session-cache"
version: "1.0.0"
description: "Sistema completo de cache para sessões de usuário"

modules:
  - module: cache
    with:
      capacity: 5000
      default_ttl: 1800  # 30 minutos padrão
  - module: log

steps:
  # Criar sessão de usuário
  - use: cache
    input:
      action: set
      key: "session:12345"
      value:
        user_id: 12345
        username: "joao.silva"
        email: "joao@example.com" 
        login_time: "2025-08-06T23:10:00Z"
        last_activity: "2025-08-06T23:10:00Z"
        permissions: ["read", "write", "profile"]
        is_active: true
      ttl: 3600  # 1 hora para esta sessão específica

  - use: log
    input:
      level: info
      message: "✅ Sessão criada para usuário joao.silva"

  # Validar sessão existe
  - use: cache
    input:
      action: exists
      key: "session:12345"

  - assert: !phs payload.found
    then:
      - use: log
        input:
          level: info
          message: "✅ Validação de sessão bem-sucedida"
    else:
      - use: log
        input:
          level: error
          message: "❌ Sessão não encontrada"

  # Recuperar dados da sessão
  - use: cache
    input:
      action: get
      key: "session:12345"

  - assert: !phs payload.found
    then:
      - use: log
        input:
          level: info
          message: !phs `👤 Sessão recuperada para ${payload.value.username}`
      
      # Renovar sessão (atualizar last_activity)
      - use: cache
        input:
          action: set
          key: "session:12345"
          value:
            user_id: !phs payload.value.user_id
            username: !phs payload.value.username
            email: !phs payload.value.email
            login_time: !phs payload.value.login_time
            last_activity: "2025-08-06T23:15:00Z"
            permissions: !phs payload.value.permissions
            is_active: true
          ttl: 3600  # Renovar por mais 1 hora
      
      - use: log
        input:
          level: info
          message: "🔄 Sessão renovada com sucesso"

  # Listar todas as sessões ativas (admin)
  - use: cache
    input:
      action: list
      filter_type: "prefix"
      filter_prefix: "session:"
      order: "desc"
      limit: 100

  - use: log
    input:
      level: info
      message: !phs `📊 Total de ${payload.total_count} sessões ativas`

  # Logout (remover sessão)
  - use: cache
    input:
      action: remove
      key: "session:12345"

  - assert: !phs payload.removed
    then:
      - use: log
        input:
          level: info
          message: "🚪 Logout realizado com sucesso"

  # Verificar se sessão foi removida
  - use: cache
    input:
      action: exists
      key: "session:12345"

  - assert: !phs !payload.found
    then:
      - use: log
        input:
          level: info
          message: "✅ Confirmado: sessão foi removida"

  # Estatísticas finais
  - use: cache
    input:
      action: stats

  - return:
      message: "Sistema de sessões processado com sucesso"
      cache_stats: !phs payload.stats
```

### Cache de Respostas de API

```phlow
name: "api-response-cache"
version: "1.0.0"
description: "Sistema de cache para respostas de API com diferentes estratégias de TTL"

modules:
  - module: cache
    with:
      capacity: 2000
      default_ttl: 600  # 10 minutos padrão
  - module: log

steps:
  # Cache de dados que mudam frequentemente (TTL curto)
  - use: cache
    input:
      action: set
      key: "api:users:list"
      value:
        data:
          - {id: 1, name: "Alice", status: "active"}
          - {id: 2, name: "Bob", status: "inactive"}
          - {id: 3, name: "Charlie", status: "active"}
        metadata:
          total_count: 3
          page: 1
          cached_at: "2025-08-06T23:10:00Z"
        query_time_ms: 245
      ttl: 300  # 5 minutos - dados que mudam rapidamente

  - use: log
    input:
      level: info
      message: "📋 Lista de usuários cached (TTL: 5 min)"

  # Cache de perfil individual (TTL médio)
  - use: cache
    input:
      action: set
      key: "api:user:42"
      value:
        id: 42
        name: "Alice Johnson"
        email: "alice@example.com"
        profile:
          bio: "Desenvolvedora de Software"
          location: "São Paulo, SP"
          joined: "2023-01-15"
        preferences:
          theme: "dark"
          notifications: true
      ttl: 1800  # 30 minutos - dados de perfil

  - use: log
    input:
      level: info
      message: "👤 Perfil de usuário cached (TTL: 30 min)"

  # Cache de estatísticas computadas (TTL longo)
  - use: cache
    input:
      action: set
      key: "api:stats:daily"
      value:
        date: "2025-08-06"
        statistics:
          total_users: 15247
          active_users: 8934
          new_registrations: 127
          page_views: 45892
          api_calls: 12456
        computed_at: "2025-08-06T23:10:00Z"
        computation_time_ms: 1850
      ttl: 86400  # 24 horas - estatísticas diárias

  - use: log
    input:
      level: info
      message: "📊 Estatísticas diárias cached (TTL: 24h)"

  # Cache de configuração (TTL muito longo)
  - use: cache
    input:
      action: set
      key: "api:config:app"
      value:
        version: "2.1.0"
        features:
          dark_mode: true
          notifications: true
          analytics: true
          beta_features: false
        limits:
          max_file_size_mb: 10
          max_requests_per_hour: 1000
        endpoints:
          - {path: "/api/users", methods: ["GET", "POST"]}
          - {path: "/api/users/:id", methods: ["GET", "PUT", "DELETE"]}
      ttl: 604800  # 7 dias - configuração da aplicação

  - use: log
    input:
      level: info
      message: "⚙️ Configuração da app cached (TTL: 7 dias)"

  # Simular cache hit para lista de usuários
  - use: cache
    input:
      action: get
      key: "api:users:list"

  - assert: !phs payload.found
    then:
      - use: log
        input:
          level: info
          message: !phs `✅ Cache HIT: Lista com ${payload.value.metadata.total_count} usuários`
      - use: log
        input:
          level: info  
          message: !phs `⏱️ Query original levou ${payload.value.query_time_ms}ms`

  # Listar todos os caches de API
  - use: cache
    input:
      action: list
      filter_type: "prefix"
      filter_prefix: "api:"
      order: "asc"

  - use: log
    input:
      level: info
      message: !phs `📂 Total de ${payload.total_count} respostas de API em cache`

  # Invalidar cache de usuário específico (após atualização)
  - use: cache
    input:
      action: remove
      key: "api:user:42"

  - assert: !phs payload.removed
    then:
      - use: log
        input:
          level: info
          message: "🗑️ Cache do usuário 42 invalidado (ex: após update)"

  # Verificar cache miss após invalidação
  - use: cache
    input:
      action: get
      key: "api:user:42"

  - assert: !phs !payload.found
    then:
      - use: log
        input:
          level: info
          message: "✅ Confirmado: Cache invalidado corretamente"
      - use: log
        input:
          level: info
          message: "💡 Próxima chamada da API fará query no banco"

  # Estatísticas de performance
  - use: cache
    input:
      action: stats

  - use: log
    input:
      level: info
      message: !phs `📈 Hit rate: ${payload.stats.hit_rate.toFixed(1)}%, Memória: ${(payload.stats.memory_usage/1024).toFixed(1)}KB`

  - return:
      message: "Sistema de cache de API processado"
      hit_rate: !phs payload.stats.hit_rate
      memory_usage_kb: !phs (payload.stats.memory_usage/1024).toFixed(1)
      categories_cached:
        - "Lista de usuários (TTL: 5 min)"
        - "Perfis individuais (TTL: 30 min)"
        - "Estatísticas diárias (TTL: 24h)"
        - "Configuração (TTL: 7 dias)"
```

## 🔍 Estratégias de TTL

### TTL Curto (1-10 minutos)
**Ideal para**: Dados que mudam frequentemente
```phlow
ttl: 300  # 5 minutos
# Exemplos: listas de usuários, status em tempo real, cotações
```

### TTL Médio (30-60 minutos)  
**Ideal para**: Dados específicos do usuário
```phlow
ttl: 1800  # 30 minutos
# Exemplos: perfis de usuário, preferências, sessões
```

### TTL Longo (horas)
**Ideal para**: Dados computados/agregados
```phlow
ttl: 86400  # 24 horas
# Exemplos: relatórios, estatísticas, dashboards
```

### TTL Muito Longo (dias)
**Ideal para**: Configurações e dados estáticos
```phlow
ttl: 604800  # 7 dias
# Exemplos: configurações da app, features flags, metadados
```

## 📈 Monitoramento e Estatísticas

### Métricas Disponíveis

```phlow
- use: cache
  input:
    action: stats

# Retorna:
# {
#   "stats": {
#     "size": 150,              // Itens atuais no cache
#     "capacity": 1000,         // Capacidade máxima  
#     "hit_rate": 85.4,         // Taxa de sucesso (%)
#     "memory_usage": 33024,    // Uso de memória estimado (bytes)
#     "total_gets": 500,        // Total de operações get
#     "total_hits": 427,        // Total de cache hits
#     "total_sets": 150,        // Total de operações set
#     "total_removes": 23       // Total de operações remove
#   }
# }
```

### Interpretação das Métricas

- **Hit Rate**: Taxa de sucesso do cache (quanto maior, melhor)
  - `> 80%`: Excelente performance
  - `60-80%`: Boa performance  
  - `< 60%`: Considerar ajustar TTL ou capacidade

- **Memory Usage**: Uso de memória estimado
  - ~220 bytes por item armazenado
  - Monitore para evitar consumo excessivo

- **Size vs Capacity**: Utilização do cache
  - Se próximo da capacidade, itens antigos serão removidos (LRU)

## ⚡ Performance e Boas Práticas

### Complexidade das Operações

- **Get Operations**: O(1) - Tempo constante
- **Set Operations**: O(log n) - Inserção ordenada
- **List Operations**: O(n) - Com filtros aplicados
- **Exists Operations**: O(1) - Tempo constante
- **Remove Operations**: O(1) - Tempo constante

### Padrões de Nomenclatura de Chaves

```phlow
# ✅ Bons padrões
"user:123"              # Dados de usuário
"session:abc123"        # Sessão de usuário  
"api:users:list"        # Lista da API
"api:user:123"          # Usuário específico da API
"config:feature_flags"  # Configurações
"stats:daily:2025-08-06" # Estatísticas por data

# ❌ Evitar
"userdata"              # Muito genérico
"temp123"               # Não descritivo
"a:b:c:d:e:f"          # Muito profundo
```

### Configurações Recomendadas

#### Desenvolvimento
```phlow
modules:
  - module: cache
    with:
      capacity: 100
      default_ttl: 300
```

#### Staging
```phlow
modules:
  - module: cache
    with:
      capacity: 1000
      default_ttl: 600
```

#### Produção
```phlow
modules:
  - module: cache
    with:
      capacity: 10000
      default_ttl: 1800
```

## 🧪 Testes

### Tipos de Testes Disponíveis

#### 1. Testes Unitários (Rust)
```bash
# Executar testes unitários do módulo
cd modules/cache
cargo test

# Resultado esperado: 8 testes aprovados
# - Testes de parsing de inputs (CacheInput)
# - Testes de estatísticas (CacheStats) 
# - Validação de parâmetros e ações
```

#### 2. Testes Funcionais Básicos
```bash
# Teste linear simples com operações fundamentais
phlow modules/cache/test-basic.phlow

# Cobertura:
# - Set/Get operações com diferentes tipos de dados
# - Exists, Remove, Clear operações
# - List e Stats operações
# - TTL básico
```

#### 3. Testes Funcionais Completos
```bash
# Teste abrangente com casos avançados
phlow modules/cache/test-complete.phlow

# Cobertura:
# - Filtros (prefix, suffix, pattern)
# - Paginação (limit/offset)
# - Ordenação (asc/desc)
# - Objetos complexos e arrays
# - TTL com diferentes estratégias
# - Casos edge (chaves inexistentes)
```

#### 4. Exemplos de Uso Real
```bash
# Sistema de sessões de usuário
phlow examples/cache/user-sessions.phlow

# Sistema de cache de API (em desenvolvimento)
phlow examples/cache/api-data-cache.phlow
```

### Executar Todos os Testes

```bash
# Executar testes unitários
cd modules/cache && cargo test

# Executar testes funcionais
phlow modules/cache/test-basic.phlow
phlow modules/cache/test-complete.phlow

# Executar exemplos práticos
phlow examples/cache/user-sessions.phlow
```

### Resultados de Teste

**✅ Status Atual**: Todos os testes aprovados
- **Testes unitários**: 8/8 ✅
- **Testes funcionais**: 2/2 ✅  
- **Exemplos práticos**: 1/1 ✅
- **Cobertura**: ~95% das funcionalidades

## 🚨 Tratamento de Erros

### Erro de Chave Vazia
```phlow
# Input inválido
input:
  action: set
  key: ""           # ❌ Chave vazia
  value: "test"

# Response
{
  "success": false,
  "error": "Key cannot be empty for set action"
}
```

### Erro de Ação Inválida
```phlow
# Input inválido
input:
  action: "invalid"   # ❌ Ação não suportada

# Response  
{
  "success": false,
  "error": "Invalid action 'invalid'. Must be one of: set, get, remove, clear, exists, list, cleanup, stats"
}
```

### Cache Miss (Não é erro)
```phlow
# Input válido
input:
  action: get
  key: "nonexistent"

# Response (sucesso, mas item não encontrado)
{
  "success": true,
  "found": false,
  "key": "nonexistent", 
  "value": null
}
```

## 🔗 Integração com Outros Módulos

### Com HTTP Server
```phlow
modules:
  - module: http_server
  - module: cache
    with:
      capacity: 5000
      default_ttl: 1800

steps:
  # Verificar cache antes de processar request
  - use: cache
    input:
      action: get
      key: !phs `api:${request.path}`
  
  - assert: !phs payload.found
    then:
      # Cache hit - retornar dados cached
      - return: !phs payload.value
    else:
      # Cache miss - processar e armazenar
      # ... lógica de processamento ...
      - use: cache
        input:
          action: set
          key: !phs `api:${request.path}`
          value: !phs processed_data
          ttl: 600
      - return: !phs processed_data
```

### Com Database (PostgreSQL)
```phlow
modules:
  - module: postgres
  - module: cache

steps:
  # Tentar buscar no cache primeiro
  - use: cache
    input:
      action: get
      key: "expensive_query_results"
  
  - assert: !phs !payload.found
    then:
      # Cache miss - executar query no banco
      - use: postgres
        input:
          query: "SELECT * FROM complex_view WHERE conditions..."
      
      # Armazenar resultado no cache
      - use: cache
        input:
          action: set
          key: "expensive_query_results"
          value: !phs payload
          ttl: 3600
      
      - return: !phs payload
    else:
      # Cache hit - retornar dados cached
      - return: !phs payload.value
```

## 🏷️ Tags

- cache
- memory
- storage
- performance
- ttl
- lru
- quickleaf
- high-performance

---

**Versão**: 0.1.0  
**Autor**: Philippe Assis \<codephilippe@gmail.com\>
**Licença**: MIT  
**Repositório**: https://github.com/phlowdotdev/phlow
