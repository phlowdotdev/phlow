# Documentação dos Testes - Módulo Cache

Esta documentação descreve os testes criados para o módulo cache do Phlow e como utilizá-los.

## Arquivos de Teste Criados

### 1. `simple-test.phlow` - Testes Básicos

Este arquivo contém 12 testes que cobrem as operações fundamentais do cache:

- **Set/Get Operations**: Armazenamento e recuperação de strings, números e objetos complexos
- **Existence Checks**: Verificação de existência de chaves
- **Remove Operations**: Remoção de itens do cache
- **List Operations**: Listagem de itens no cache
- **Statistics**: Obtenção de estatísticas do cache
- **Clear Operations**: Limpeza completa do cache

#### Como executar:

```bash
# Executar todos os testes simples
phlow --test modules/cache/simple-test.phlow

# Executar apenas testes relacionados a strings
phlow --test --test-filter "string" modules/cache/simple-test.phlow

# Executar apenas testes de objetos
phlow --test --test-filter "object" modules/cache/simple-test.phlow
```

#### Resultado esperado:
```
🧪 Running 12 test(s)...

Test 1: Set and get string value - ✅ PASSED
Test 2: Retrieve stored string value - ✅ PASSED
Test 3: Set and get number value - ✅ PASSED
Test 4: Retrieve stored number value - ✅ PASSED
Test 5: Set complex object - ✅ PASSED
Test 6: Retrieve stored object - ✅ PASSED
Test 7: Check if key exists - ✅ PASSED
Test 8: Check non-existent key - ✅ PASSED
Test 9: Remove existing key - ✅ PASSED
Test 10: List all cached items - ✅ PASSED
Test 11: Get cache statistics - ✅ PASSED
Test 12: Clear entire cache - ✅ PASSED

📊 Test Results:
   Total: 12
   Passed: 12 ✅
   Failed: 0 ❌

🎉 All tests passed!
```

### 2. `test.phlow` - Testes Abrangentes

Este arquivo contém 23 testes mais detalhados que incluem:

- **Operações com TTL**: Testes de Time-To-Live customizado
- **Filtragem Avançada**: Testes de filtros por prefixo, sufixo e padrões
- **Paginação**: Testes de listagem com limite e offset
- **Ordenação**: Testes de ordenação ascendente e descendente
- **Condições de Erro**: Testes de tratamento de erros
- **Limpeza Manual**: Testes de cleanup de itens expirados

#### Como executar:

```bash
# Executar todos os testes abrangentes
phlow --test modules/cache/test.phlow

# Executar apenas testes de TTL
phlow --test --test-filter "TTL" modules/cache/test.phlow

# Executar apenas testes de paginação
phlow --test --test-filter "pagination" modules/cache/test.phlow
```

## Estrutura dos Testes

### Formato dos Testes

Cada teste segue a estrutura do sistema de testes do Phlow:

```phlow
tests:
  - describe: "Descrição legível do teste"
    main:
      # Dados de entrada para o teste
      key: "valor"
    payload: null
    assert: !phs condição_de_verificação

steps:
  # Implementação dos steps que executam as operações do cache
  - assert: !phs condição_para_executar_step
    then:
      - use: cache
        input:
          action: operação
          key: !phs main.key
          # outros parâmetros
```

### Tipos de Assertivas

1. **`assert`**: Expressões PHS que retornam boolean
   ```phlow
   assert: !phs payload.success && payload.found
   ```

2. **`assert_eq`**: Comparação direta de valores
   ```phlow
   assert_eq: "valor esperado"
   ```

### Operações Testadas

#### 1. Set Operations
- Armazenamento com TTL customizado
- Armazenamento com TTL padrão
- Armazenamento de diferentes tipos de dados

#### 2. Get Operations
- Recuperação de dados existentes (cache hit)
- Tentativa de recuperação de dados inexistentes (cache miss)

#### 3. Exists Operations
- Verificação de existência de chaves válidas
- Verificação de chaves inexistentes

#### 4. Remove Operations
- Remoção de chaves existentes
- Tentativa de remoção de chaves inexistentes

#### 5. List Operations
- Listagem completa
- Filtragem por prefixo
- Filtragem por sufixo
- Paginação com limit/offset
- Ordenação ascendente/descendente

#### 6. Stats Operations
- Obtenção de estatísticas do cache
- Verificação de hit rate
- Verificação de capacidade e uso

#### 7. Clear Operations
- Limpeza completa do cache
- Verificação do número de itens removidos

#### 8. Cleanup Operations
- Limpeza manual de itens expirados

## Configuração dos Testes

### Configuração do Módulo Cache nos Testes

```phlow
modules:
  - module: cache
    with:
      capacity: 10          # Capacidade pequena para testes
      default_ttl: 300      # TTL padrão de 5 minutos
      enable_events: false  # Eventos desabilitados para simplicidade
```

### Dados de Teste

Os testes utilizam dados representativos:

- **Strings simples**: "Hello Cache!"
- **Números**: 42
- **Objetos complexos**:
  ```json
  {
    "name": "Test User",
    "age": 30,
    "active": true
  }
  ```
- **Arrays**: [1, 2, 3, 4, 5]

## Debugging de Testes

### Verificar Status Individual

Para debugar um teste específico, você pode executar o fluxo normalmente:

```bash
# Executar o fluxo para ver o payload real
phlow modules/cache/simple-test.phlow
```

### Usar Filtros de Teste

```bash
# Testar apenas operações específicas
phlow --test --test-filter "set" modules/cache/simple-test.phlow
phlow --test --test-filter "get" modules/cache/simple-test.phlow
phlow --test --test-filter "remove" modules/cache/simple-test.phlow
```

## Melhores Práticas

### 1. Nomenclatura Descritiva
Cada teste tem uma descrição clara do que está sendo testado:
```phlow
- describe: "Set and get string value"
- describe: "Retrieve stored string value"  
- describe: "Check non-existent key"
```

### 2. Isolamento de Testes
Cada teste é independente e pode ser executado isoladamente.

### 3. Cobertura Abrangente
Os testes cobrem:
- ✅ Casos de sucesso
- ✅ Casos de erro
- ✅ Condições limite
- ✅ Diferentes tipos de dados
- ✅ Todas as operações do módulo

### 4. Verificações Específicas
Cada teste verifica aspectos específicos:
```phlow
# Verifica tanto sucesso quanto conteúdo
assert: !phs payload.found && payload.value == "Hello Cache!"

# Verifica estrutura de objetos
assert: !phs payload.value.name == "Test User" && payload.value.age == 30
```

## Integração com CI/CD

Os testes podem ser integrados em pipelines de CI/CD:

```bash
#!/bin/bash
# Script de teste automatizado

echo "Executando testes básicos do cache..."
if ! phlow --test modules/cache/simple-test.phlow; then
    echo "Testes básicos falharam!"
    exit 1
fi

echo "Executando testes abrangentes do cache..."
if ! phlow --test modules/cache/test.phlow; then
    echo "Testes abrangentes falharam!"
    exit 1
fi

echo "Todos os testes do cache passaram com sucesso!"
```

## Próximos Passos

1. **Testes de Performance**: Criar testes para verificar performance com alta carga
2. **Testes de TTL**: Criar testes que verificam expiração real de itens
3. **Testes de Eventos**: Criar testes para funcionalidade de eventos quando habilitada
4. **Testes de Concorrência**: Criar testes para operações simultâneas

---

## Comandos de Referência Rápida

```bash
# Executar todos os testes básicos
phlow --test modules/cache/simple-test.phlow

# Executar testes específicos com filtro
phlow --test --test-filter "string" modules/cache/simple-test.phlow

# Executar exemplo completo
phlow modules/cache/example.phlow

# Verificar se o módulo está compilado
ls -la phlow_packages/cache/

# Recompilar módulo se necessário
cargo make local cache
```
