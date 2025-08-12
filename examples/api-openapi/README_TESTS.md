# Scripts de Teste da API OpenAPI

Este diretório contém scripts bash para testar completamente a API OpenAPI implementada no módulo http_server do Phlow.

## Scripts Disponíveis

### 1. `test_api.sh` - Teste Completo
Script completo que testa todas as rotas e cenários possíveis da API OpenAPI.

**Características:**
- ✅ 38 testes abrangentes
- ✅ Testa todas as rotas: GET, POST, PUT, DELETE
- ✅ Valida códigos de status HTTP corretos
- ✅ Testa validações OpenAPI (campos obrigatórios, tipos, padrões regex)
- ✅ Testa cenários de erro (404, 400, 405)
- ✅ Output colorido e estatísticas finais
- ✅ Demonstra que o bug de validação PUT foi corrigido

**Uso:**
```bash
./test_api.sh
```

### 2. `quick_test.sh` - Teste Rápido
Script simplificado para validação rápida das funcionalidades principais.

**Características:**
- ⚡ Teste rápido com cenários essenciais
- ✅ Foca no bug corrigido (PUT sem campos obrigatórios)
- ✅ Valida operações CRUD básicas
- ✅ Output conciso e direto

**Uso:**
```bash
./quick_test.sh
```

## Pré-requisitos

1. **Servidor rodando**: Certifique-se de que o servidor está rodando em `http://localhost:8080`
2. **curl instalado**: Os scripts usam curl para fazer as requisições HTTP

## Como iniciar o servidor

```bash
cd /home/assis/projects/lowcarboncode/phlow/examples/api-openapi
../../target/release/phlow-runtime run main.phlow
```

Ou se ainda não compilou:
```bash
cd /home/assis/projects/lowcarboncode/phlow
cargo build --release
cd examples/api-openapi
../../target/release/phlow-runtime run main.phlow
```

## Cenários de Teste Cobertos

### POST /users (Criar usuário)
- ✅ **201 Created**: Usuário válido com todos os campos
- ✅ **201 Created**: Usuário com campos mínimos obrigatórios (name, email)
- ✅ **201 Created**: Usuário com propriedades adicionais permitidas
- ❌ **400 Bad Request**: Falta campo obrigatório (name ou email)
- ❌ **400 Bad Request**: Validação de formato (nome com números, email inválido)
- ❌ **400 Bad Request**: Validação de tamanho (nome muito curto/longo)
- ❌ **400 Bad Request**: Validação de range (idade negativa/muito alta)
- ❌ **400 Bad Request**: Corpo vazio ou JSON malformado

### GET /users (Listar usuários)
- ✅ **200 OK**: Lista todos os usuários

### GET /users/{userId} (Obter usuário)
- ✅ **200 OK**: Usuário existente
- ❌ **404 Not Found**: Usuário inexistente ou ID inválido

### PUT /users/{userId} (Atualizar usuário) - **BUG CORRIGIDO**
- ✅ **200 OK**: Atualização com dados válidos (sem exigir campos obrigatórios)
- ✅ **200 OK**: Atualização parcial (apenas nome, apenas idade, etc.)
- ✅ **200 OK**: Atualização com propriedades extras
- ❌ **400 Bad Request**: Dados inválidos (nome muito curto, idade negativa, etc.)
- ❌ **404 Not Found**: Usuário inexistente

### DELETE /users/{userId} (Deletar usuário)
- ✅ **204 No Content**: Usuário deletado com sucesso
- ❌ **404 Not Found**: Usuário inexistente

### Métodos Não Permitidos
- ❌ **405 Method Not Allowed**: PATCH, DELETE em /users
- ❌ **405 Method Not Allowed**: POST, PATCH em /users/{userId}

### Rotas Inexistentes
- ❌ **404 Not Found**: Qualquer rota não definida no OpenAPI

## Bug Corrigido

O principal bug corrigido foi que requisições **PUT** estavam sendo validadas usando o schema do **POST**, o que causava:

❌ **Antes da correção:**
```bash
curl -X PUT http://localhost:8080/users/123 -d '{"age":30}' 
# Retornava 400 - campos 'name' e 'email' são obrigatórios
```

✅ **Após a correção:**
```bash
curl -X PUT http://localhost:8080/users/123 -d '{"age":30}'
# Retorna 200 - atualização parcial permitida
```

## Interpretando os Resultados

### Teste Completo (`test_api.sh`)
```
✓ PASS - Criar usuário válido (Expected: 201, Got: 201)
✗ FAIL - Criar usuário sem nome (Expected: 400, Got: 201)

ESTATÍSTICAS FINAIS
Total de testes executados: 38
Testes aprovados: 36
Testes falharam: 2
```

### Teste Rápido (`quick_test.sh`)
```
✓ Criar usuário válido (201)
✗ Criar usuário sem nome (esperado: 400, obtido: 201)
✓ PUT com apenas idade (bug corrigido) (200)
```

## Validações OpenAPI Testadas

1. **Campos Obrigatórios**: `name` e `email` são obrigatórios apenas no POST
2. **Tipos de Campo**: string, integer conforme definido no schema
3. **Validação de Tamanho**: minLength=2, maxLength=50 para nome
4. **Validação de Pattern**: regex para nome (apenas letras) e email
5. **Validação de Range**: idade entre 0-120
6. **Propriedades Adicionais**: permitidas (`additionalProperties: true`)
7. **Formato Built-in**: validação de email format

## Extensão dos Testes

Para adicionar novos testes, edite os arquivos e siga o padrão:

```bash
test_request "MÉTODO" "$BASE_URL/rota" "status_esperado" \
    "Descrição do teste" \
    '{"dados": "json"}'  # apenas se precisar de body
```

Os scripts são facilmente extensíveis para testar novas rotas conforme você adicionar ao `openapi.yaml`.
