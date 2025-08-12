# 🧪 Testes de Integração - API OpenAPI

Este diretório contém scripts completos de teste de integração para validar a funcionalidade da API OpenAPI implementada com Phlow.

## 📋 Scripts Disponíveis

### 1. `integration_test.sh` - Script Principal de Testes
Script completo que executa todos os cenários de teste de integração da API.

**Execução manual:**
```bash
# Certifique-se de que o servidor está rodando
PHLOW_LOG=debug phlow examples/api-openapi &

# Execute os testes
./examples/api-openapi/integration_test.sh
```

### 2. `run_integration_tests.sh` - Execução Automatizada
Script auxiliar que automatiza todo o processo: build, inicialização do servidor e execução dos testes.

**Execução automatizada (recomendada):**
```bash
./examples/api-openapi/run_integration_tests.sh
```

## 🎯 Cenários de Teste Cobertos

### **A. Conectividade e Sistema**
- ✅ Health check (`/health`)
- ✅ Endpoint OpenAPI spec (`/openapi.json`)
- ✅ Conectividade básica

### **B. POST /users - Criação de Usuários**

#### **B.1 Casos Válidos:**
- ✅ Usuário com dados completos
- ✅ Usuário com dados mínimos obrigatórios  
- ✅ Usuário com propriedades adicionais

#### **B.2 Validação de Campos Obrigatórios:**
- ❌ Sem campo `name`
- ❌ Sem campo `email`
- ❌ Sem ambos os campos
- ❌ Body vazio
- ❌ Sem body

#### **B.3 Validação de Nome:**
- ❌ Nome muito curto (< 2 caracteres)
- ❌ Nome muito longo (> 50 caracteres)
- ❌ Nome com números
- ❌ Nome com caracteres especiais inválidos
- ✅ Nome válido com espaços

#### **B.4 Validação de Email:**
- ❌ Email sem @
- ❌ Email sem domínio
- ❌ Email sem TLD
- ✅ Email com formato válido
- ✅ Email com subdomínio

#### **B.5 Validação de Idade:**
- ❌ Idade negativa
- ❌ Idade muito alta (> 120)
- ❌ Idade como string
- ✅ Idade válida

#### **B.6 Validação de Telefone:**
- ❌ Formato inválido
- ✅ Formato válido internacional
- ✅ Formato válido nacional

### **C. GET /users - Listagem**
- ✅ Listar usuários

### **D. GET /users/{userId} - Busca Individual**
- ✅ Usuário existente
- ❌ Usuário inexistente
- ❌ ID inválido

### **E. PUT /users/{userId} - Atualização**
- ✅ Atualizar usuário existente
- ✅ Atualização parcial
- ❌ Usuário inexistente
- ❌ Dados inválidos na atualização

### **F. DELETE /users/{userId} - Remoção**
- ✅ Deletar usuário existente
- ❌ Usuário inexistente
- ❌ ID inválido

### **G. Métodos Não Permitidos**
- ❌ PATCH em rotas não suportadas
- ❌ Métodos HTTP incorretos

### **H. Rotas Inexistentes**
- ❌ Rotas que não existem
- ❌ Subrotas inexistentes

### **I. Formato e Headers**
- ❌ JSON malformado
- ❌ Content-Type incorreto
- ❌ Dados inválidos

### **J. Edge Cases**
- ❌ Valores limítrofes inválidos
- ✅ Valores limítrofes válidos
- ❌ Múltiplos campos inválidos

## 📊 Relatório de Testes

Após a execução, você verá:

```bash
============================================================
RELATÓRIO FINAL DE TESTES DE INTEGRAÇÃO  
============================================================

Estatísticas Gerais:
  Total de testes executados: XX
  Testes aprovados: XX
  Testes falharam: XX
  Taxa de sucesso: XX%

🎉 TODOS OS TESTES PASSARAM!
✅ A API está funcionando corretamente em todos os cenários testados.
```

## 🔧 Configuração

### **Pré-requisitos:**
- Phlow instalado e configurado
- `curl` disponível no sistema
- Porta 3000 livre (ou modificar `BASE_URL` nos scripts)

### **Variáveis Configuráveis:**
```bash
# No integration_test.sh
BASE_URL="http://localhost:3000"    # URL base da API
TIMEOUT=5                           # Timeout para requisições
```

## 🐛 Troubleshooting

### **Servidor não responde:**
```bash
# Verificar se porta está em uso
lsof -ti:3000

# Matar processos na porta
pkill -f "phlow examples/api-openapi"
```

### **Testes falhando:**
```bash
# Ver logs detalhados do servidor
tail -f server_test.log

# Executar teste individual
curl -X POST -H "Content-Type: application/json" \
  -d '{"name":"Teste","email":"teste@exemplo.com"}' \
  http://localhost:3000/users
```

### **Problemas de permissão:**
```bash
chmod +x integration_test.sh
chmod +x run_integration_tests.sh
```

## 📁 Arquivos Relacionados

- `integration_test.sh` - Script principal de testes
- `run_integration_tests.sh` - Script de execução automatizada
- `openapi.yaml` - Especificação OpenAPI da API
- `main.phlow` - Fluxo principal da aplicação
- `server_test.log` - Logs do servidor durante testes

## 🚀 Uso em CI/CD

Para integração contínua, use o script automatizado:

```yaml
# Exemplo para GitHub Actions
- name: Run Integration Tests
  run: ./examples/api-openapi/run_integration_tests.sh
```

O script retorna código de saída 0 para sucesso e 1 para falha, compatível com sistemas de CI/CD.

## 📝 Contribuição

Para adicionar novos testes:

1. Adicione o cenário no `integration_test.sh`
2. Use a função `test_request()` existente
3. Adicione na seção apropriada (A-J)
4. Atualize esta documentação

**Exemplo:**
```bash
test_request "POST" "$BASE_URL/users" "400" \
    "Descrição do teste" \
    '{"campo":"valor_teste"}'
```

---

**📧 Suporte:** Para questões sobre os testes, verifique os logs e a documentação da API OpenAPI.
