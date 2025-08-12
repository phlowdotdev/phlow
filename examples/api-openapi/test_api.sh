#!/bin/bash

# Script de teste completo para API OpenAPI
# Testa todas as rotas com validação de códigos de status HTTP

# Configurações
BASE_URL="http://localhost:3000"
CONTENT_TYPE="Content-Type: application/json"

# Cores para output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Contadores para estatísticas
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Função para imprimir cabeçalho
print_header() {
    echo -e "\n${BLUE}================================"
    echo -e "$1"
    echo -e "================================${NC}\n"
}

# Função para imprimir resultado do teste
print_result() {
    local test_name="$1"
    local expected_status="$2"
    local actual_status="$3"
    local success="$4"
    
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    if [ "$success" = "true" ]; then
        echo -e "${GREEN}✓ PASS${NC} - $test_name (Expected: $expected_status, Got: $actual_status)"
        PASSED_TESTS=$((PASSED_TESTS + 1))
    else
        echo -e "${RED}✗ FAIL${NC} - $test_name (Expected: $expected_status, Got: $actual_status)"
        FAILED_TESTS=$((FAILED_TESTS + 1))
    fi
}

# Função para fazer requisição HTTP e validar status
test_request() {
    local method="$1"
    local url="$2"
    local expected_status="$3"
    local test_name="$4"
    local data="$5"
    
    if [ -n "$data" ]; then
        response=$(curl -s -w "\n%{http_code}" -X "$method" -H "$CONTENT_TYPE" -d "$data" "$url")
    else
        response=$(curl -s -w "\n%{http_code}" -X "$method" "$url")
    fi
    
    actual_status=$(echo "$response" | tail -n1)
    response_body=$(echo "$response" | head -n -1)
    
    # Verificar se o status obtido está na lista de status esperados
    if echo "$expected_status" | grep -q "$actual_status"; then
        print_result "$test_name" "$expected_status" "$actual_status" "true"
        if [ -n "$response_body" ] && [ "$response_body" != "{}" ]; then
            echo -e "  ${YELLOW}Response:${NC} $response_body"
        fi
    else
        print_result "$test_name" "$expected_status" "$actual_status" "false"
        if [ -n "$response_body" ]; then
            echo -e "  ${YELLOW}Response:${NC} $response_body"
        fi
    fi
    
    echo
}

# Verificar se o servidor está rodando
check_server() {
    echo -e "${BLUE}Verificando se o servidor está rodando...${NC}"
    if ! curl -s -f "$BASE_URL/health" >/dev/null 2>&1; then
        if ! curl -s -f "$BASE_URL" >/dev/null 2>&1; then
            echo -e "${RED}Erro: Servidor não está respondendo em $BASE_URL${NC}"
            echo "Certifique-se de que o servidor está rodando."
            exit 1
        fi
    fi
    echo -e "${GREEN}Servidor está rodando!${NC}\n"
}

# Função para aguardar entre testes
sleep_between_tests() {
    sleep 0.5  # Pequena pausa entre testes
}

print_header "INICIANDO TESTES DA API OpenAPI"

check_server

# =============================================================================
# TESTES PARA /users (POST - Criar usuário)
# =============================================================================

print_header "TESTANDO POST /users - Criar Usuário"

# Teste 1: Criar usuário válido (deve retornar 201)
test_request "POST" "$BASE_URL/users" "201" \
    "Criar usuário válido" \
    '{"name":"João Silva","email":"joao@email.com","age":30}'

sleep_between_tests

# Teste 2: Criar usuário com dados mínimos obrigatórios (deve retornar 201)
test_request "POST" "$BASE_URL/users" "201" \
    "Criar usuário com dados mínimos" \
    '{"name":"Maria Santos","email":"maria@email.com"}'

sleep_between_tests

# Teste 3: Criar usuário com propriedades adicionais (deve retornar 201)
test_request "POST" "$BASE_URL/users" "201" \
    "Criar usuário com propriedades extras" \
    '{"name":"Ana Costa","email":"ana@email.com","age":25,"phone":"+5511999999999","city":"São Paulo"}'

sleep_between_tests

# Teste 4: Criar usuário sem nome (deve retornar 400)
test_request "POST" "$BASE_URL/users" "400" \
    "Criar usuário sem nome (campo obrigatório)" \
    '{"email":"teste@email.com","age":25}'

sleep_between_tests

# Teste 5: Criar usuário sem email (deve retornar 400)
test_request "POST" "$BASE_URL/users" "400" \
    "Criar usuário sem email (campo obrigatório)" \
    '{"name":"Teste User","age":25}'

sleep_between_tests

# Teste 6: Criar usuário com nome muito curto (deve retornar 400)
test_request "POST" "$BASE_URL/users" "400" \
    "Criar usuário com nome muito curto" \
    '{"name":"A","email":"teste@email.com"}'

sleep_between_tests

# Teste 7: Criar usuário com nome muito longo (deve retornar 400)
test_request "POST" "$BASE_URL/users" "400" \
    "Criar usuário com nome muito longo" \
    '{"name":"'"$(printf '%0.s' {1..60} | tr '0' 'A')"'","email":"teste@email.com"}'

sleep_between_tests

# Teste 8: Criar usuário com nome contendo números (deve retornar 400)
test_request "POST" "$BASE_URL/users" "400" \
    "Criar usuário com nome inválido (números)" \
    '{"name":"João123","email":"teste@email.com"}'

sleep_between_tests

# Teste 9: Criar usuário com email inválido (deve retornar 400)
test_request "POST" "$BASE_URL/users" "400" \
    "Criar usuário com email inválido" \
    '{"name":"Teste User","email":"email-inválido"}'

sleep_between_tests

# Teste 10: Criar usuário com idade negativa (deve retornar 400)
test_request "POST" "$BASE_URL/users" "400" \
    "Criar usuário com idade negativa" \
    '{"name":"Teste User","email":"teste@email.com","age":-5}'

sleep_between_tests

# Teste 11: Criar usuário com idade muito alta (deve retornar 400)
test_request "POST" "$BASE_URL/users" "400" \
    "Criar usuário com idade muito alta" \
    '{"name":"Teste User","email":"teste@email.com","age":150}'

sleep_between_tests

# Teste 12: Criar usuário com telefone inválido (deve retornar 400)
test_request "POST" "$BASE_URL/users" "400" \
    "Criar usuário com telefone inválido" \
    '{"name":"Teste User","email":"teste@email.com","phone":"123abc"}'

sleep_between_tests

# Teste 13: Criar usuário com corpo vazio (deve retornar 400)
test_request "POST" "$BASE_URL/users" "400" \
    "Criar usuário com corpo vazio" \
    '{}'

sleep_between_tests

# Teste 14: Criar usuário com JSON malformado (deve retornar 400)
test_request "POST" "$BASE_URL/users" "400" \
    "Criar usuário com JSON inválido" \
    '{"name":"Teste",'

sleep_between_tests

# =============================================================================
# TESTES PARA /users (GET - Listar usuários)
# =============================================================================

print_header "TESTANDO GET /users - Listar Usuários"

# Teste 15: Listar todos os usuários (deve retornar 200)
test_request "GET" "$BASE_URL/users" "200" \
    "Listar todos os usuários"

sleep_between_tests

# =============================================================================
# TESTES PARA /users/{userId} (GET - Obter usuário específico)
# =============================================================================

print_header "TESTANDO GET /users/{userId} - Obter Usuário por ID"

# Teste 16: Obter usuário existente (deve retornar 200)
test_request "GET" "$BASE_URL/users/123" "200" \
    "Obter usuário existente"

sleep_between_tests

# Teste 17: Obter usuário inexistente (deve retornar 404)
test_request "GET" "$BASE_URL/users/999999" "404" \
    "Obter usuário inexistente"

sleep_between_tests

# Teste 18: Obter usuário com ID inválido (deve retornar 404)
test_request "GET" "$BASE_URL/users/abc" "404" \
    "Obter usuário com ID inválido"

sleep_between_tests

# =============================================================================
# TESTES PARA /users/{userId} (PUT - Atualizar usuário)
# =============================================================================

print_header "TESTANDO PUT /users/{userId} - Atualizar Usuário"

# Teste 19: Atualizar usuário existente com dados válidos (deve retornar 200)
test_request "PUT" "$BASE_URL/users/123" "200" \
    "Atualizar usuário existente" \
    '{"name":"João Silva Atualizado","age":31}'

sleep_between_tests

# Teste 20: Atualizar usuário com apenas nome (deve retornar 200)
test_request "PUT" "$BASE_URL/users/123" "200" \
    "Atualizar apenas nome do usuário" \
    '{"name":"Novo Nome"}'

sleep_between_tests

# Teste 21: Atualizar usuário com apenas idade (deve retornar 200)
test_request "PUT" "$BASE_URL/users/123" "200" \
    "Atualizar apenas idade do usuário" \
    '{"age":32}'

sleep_between_tests

# Teste 22: Atualizar usuário com propriedades extras (deve retornar 200)
test_request "PUT" "$BASE_URL/users/123" "200" \
    "Atualizar usuário com propriedades extras" \
    '{"name":"Nome Extra","age":33,"city":"Rio de Janeiro","country":"Brasil"}'

sleep_between_tests

# Teste 23: Atualizar usuário inexistente (deve retornar 404)
test_request "PUT" "$BASE_URL/users/999999" "404" \
    "Atualizar usuário inexistente" \
    '{"name":"Teste","age":25}'

sleep_between_tests

# Teste 24: Atualizar usuário com nome inválido (deve retornar 400)
test_request "PUT" "$BASE_URL/users/123" "400" \
    "Atualizar com nome inválido" \
    '{"name":"123ABC"}'

sleep_between_tests

# Teste 25: Atualizar usuário com nome muito curto (deve retornar 400)
test_request "PUT" "$BASE_URL/users/123" "400" \
    "Atualizar com nome muito curto" \
    '{"name":"A"}'

sleep_between_tests

# Teste 26: Atualizar usuário com idade negativa (deve retornar 400)
test_request "PUT" "$BASE_URL/users/123" "400" \
    "Atualizar com idade negativa" \
    '{"age":-1}'

sleep_between_tests

# Teste 27: Atualizar usuário com idade muito alta (deve retornar 400)
test_request "PUT" "$BASE_URL/users/123" "400" \
    "Atualizar com idade muito alta" \
    '{"age":999}'

sleep_between_tests

# Teste 28: Atualizar usuário com telefone inválido (deve retornar 400)
test_request "PUT" "$BASE_URL/users/123" "400" \
    "Atualizar com telefone inválido" \
    '{"phone":"abc123"}'

sleep_between_tests

# Teste 29: Atualizar usuário com corpo vazio (deve retornar 400)
test_request "PUT" "$BASE_URL/users/123" "400" \
    "Atualizar com corpo vazio" \
    '{}'

sleep_between_tests

# =============================================================================
# TESTES PARA /users/{userId} (DELETE - Deletar usuário)
# =============================================================================

print_header "TESTANDO DELETE /users/{userId} - Deletar Usuário"

# Teste 30: Deletar usuário existente (deve retornar 204)
test_request "DELETE" "$BASE_URL/users/456" "204" \
    "Deletar usuário existente"

sleep_between_tests

# Teste 31: Deletar usuário inexistente (deve retornar 404)
test_request "DELETE" "$BASE_URL/users/999999" "404" \
    "Deletar usuário inexistente"

sleep_between_tests

# Teste 32: Deletar usuário com ID inválido (deve retornar 404)
test_request "DELETE" "$BASE_URL/users/abc" "404" \
    "Deletar usuário com ID inválido"

sleep_between_tests

# =============================================================================
# TESTES DE MÉTODOS NÃO PERMITIDOS
# =============================================================================

print_header "TESTANDO MÉTODOS NÃO PERMITIDOS"

# Teste 33: PATCH em /users (deve retornar 405)
test_request "PATCH" "$BASE_URL/users" "405" \
    "PATCH não permitido em /users"

sleep_between_tests

# Teste 34: DELETE em /users (deve retornar 405)
test_request "DELETE" "$BASE_URL/users" "405" \
    "DELETE não permitido em /users"

sleep_between_tests

# Teste 35: POST em /users/{userId} (deve retornar 405)
test_request "POST" "$BASE_URL/users/123" "405" \
    "POST não permitido em /users/{userId}" \
    '{"name":"Teste"}'

sleep_between_tests

# Teste 36: PATCH em /users/{userId} (deve retornar 405)
test_request "PATCH" "$BASE_URL/users/123" "405" \
    "PATCH não permitido em /users/{userId}" \
    '{"name":"Teste"}'

sleep_between_tests

# =============================================================================
# TESTES DE ROTAS INEXISTENTES
# =============================================================================

print_header "TESTANDO ROTAS INEXISTENTES"

# Teste 37: Rota inexistente (deve retornar 404)
test_request "GET" "$BASE_URL/rota-inexistente" "404" \
    "Acessar rota inexistente"

sleep_between_tests

# Teste 38: Subrota inexistente (deve retornar 404)
test_request "GET" "$BASE_URL/users/123/inexistente" "404" \
    "Acessar subrota inexistente"

sleep_between_tests

# =============================================================================
# ESTATÍSTICAS FINAIS
# =============================================================================

print_header "ESTATÍSTICAS FINAIS"

echo -e "${BLUE}Total de testes executados:${NC} $TOTAL_TESTS"
echo -e "${GREEN}Testes aprovados:${NC} $PASSED_TESTS"
echo -e "${RED}Testes falharam:${NC} $FAILED_TESTS"

if [ $FAILED_TESTS -eq 0 ]; then
    echo -e "\n${GREEN}🎉 TODOS OS TESTES PASSARAM!${NC}"
    exit 0
else
    FAILURE_RATE=$((FAILED_TESTS * 100 / TOTAL_TESTS))
    echo -e "\n${RED}❌ $FAILED_TESTS teste(s) falharam (${FAILURE_RATE}% de falha)${NC}"
    exit 1
fi
