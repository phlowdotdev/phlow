#!/bin/bash

# Script de teste rápido para verificar funcionalidade básica da API
# Testa os cenários mais importantes rapidamente

BASE_URL="http://localhost:3000"
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}🚀 TESTE RÁPIDO DA API OpenAPI${NC}\n"

# Verificar se servidor está rodando
if ! curl -s "$BASE_URL" >/dev/null 2>&1; then
    echo -e "${RED}❌ Servidor não está rodando em $BASE_URL${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Servidor está rodando${NC}\n"

# Função para teste rápido
quick_test() {
    local method="$1"
    local url="$2"
    local expected="$3"
    local desc="$4"
    local data="$5"
    
    if [ -n "$data" ]; then
        status=$(curl -s -w "%{http_code}" -X "$method" -H "Content-Type: application/json" -d "$data" -o /dev/null "$url")
    else
        status=$(curl -s -w "%{http_code}" -X "$method" -o /dev/null "$url")
    fi
    
    if echo "$expected" | grep -q "$status"; then
        echo -e "${GREEN}✓${NC} $desc (${status})"
    else
        echo -e "${RED}✗${NC} $desc (esperado: $expected, obtido: $status)"
    fi
}

echo -e "${YELLOW}📝 TESTANDO OPERAÇÕES PRINCIPAIS:${NC}\n"

# Testes essenciais
quick_test "POST" "$BASE_URL/users" "201" "Criar usuário válido" \
    '{"name":"João Teste","email":"joao@teste.com","age":30}'

quick_test "POST" "$BASE_URL/users" "400" "Criar usuário sem nome (deve falhar)" \
    '{"email":"teste@teste.com"}'

quick_test "POST" "$BASE_URL/users" "400" "Criar usuário com nome inválido (deve falhar)" \
    '{"name":"João123","email":"teste@teste.com"}'

quick_test "GET" "$BASE_URL/users" "200" "Listar usuários"

quick_test "GET" "$BASE_URL/users/123" "200" "Obter usuário por ID"

quick_test "GET" "$BASE_URL/users/999999" "404" "Obter usuário inexistente (deve falhar)"

quick_test "PUT" "$BASE_URL/users/123" "200" "Atualizar usuário (sem campos obrigatórios)" \
    '{"age":35}'

quick_test "PUT" "$BASE_URL/users/123" "400" "Atualizar com dados inválidos (deve falhar)" \
    '{"name":"A"}'

quick_test "PUT" "$BASE_URL/users/999999" "404" "Atualizar usuário inexistente (deve falhar)" \
    '{"name":"Teste"}'

quick_test "DELETE" "$BASE_URL/users/456" "204" "Deletar usuário"

quick_test "DELETE" "$BASE_URL/users/999999" "404" "Deletar usuário inexistente (deve falhar)"

# Testes de método não permitido
quick_test "PATCH" "$BASE_URL/users" "405" "Método PATCH não permitido"

quick_test "POST" "$BASE_URL/users/123" "405" "POST não permitido em /users/{id}"

# Teste de rota inexistente
quick_test "GET" "$BASE_URL/inexistente" "404" "Rota inexistente"

echo -e "\n${BLUE}🎯 TESTE ESPECÍFICO DO BUG CORRIGIDO:${NC}"
echo -e "Verificando se PUT aceita atualizações parciais sem campos obrigatórios...\n"

# Este é o teste específico do bug que corrigimos
quick_test "PUT" "$BASE_URL/users/123" "200" "PUT com apenas idade (bug corrigido)" \
    '{"age":40}'

quick_test "PUT" "$BASE_URL/users/123" "200" "PUT com apenas nome (bug corrigido)" \
    '{"name":"Nome Atualizado"}'

quick_test "PUT" "$BASE_URL/users/123" "200" "PUT com propriedades extras (bug corrigido)" \
    '{"city":"São Paulo","country":"Brasil"}'

echo -e "\n${GREEN}✅ TESTE RÁPIDO CONCLUÍDO!${NC}"
echo -e "Para testes mais detalhados, execute: ${YELLOW}./test_api.sh${NC}"
