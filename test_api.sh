#!/bin/bash

# Cores para output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# URL base da API
API_URL="http://localhost:3000"

echo -e "${BLUE}=== Testando API OpenAPI - Validação de Usuários ===${NC}\n"

# Função para fazer requisições e mostrar resultado
test_request() {
    local title="$1"
    local method="$2"
    local url="$3"
    local data="$4"
    local expected_status="$5"
    
    echo -e "${YELLOW}--- $title ---${NC}"
    echo "Request: $method $url"
    if [ ! -z "$data" ]; then
        echo "Data: $data"
    fi
    
    if [ ! -z "$data" ]; then
        response=$(curl -s -w "\nHTTP_STATUS:%{http_code}" \
            -X "$method" \
            -H "Content-Type: application/json" \
            -H "Authorization: Bearer test-token" \
            -H "X-API-Key: test-api-key" \
            -d "$data" \
            "$url")
    else
        response=$(curl -s -w "\nHTTP_STATUS:%{http_code}" \
            -X "$method" \
            -H "Content-Type: application/json" \
            -H "Authorization: Bearer test-token" \
            -H "X-API-Key: test-api-key" \
            "$url")
    fi
    
    # Separar body e status code
    body=$(echo "$response" | sed '$d')
    status_code=$(echo "$response" | grep "HTTP_STATUS:" | cut -d: -f2)
    
    echo "Status Code: $status_code"
    echo "Response Body:"
    echo "$body" | jq . 2>/dev/null || echo "$body"
    
    # Verificar se o status code está correto
    if [ "$status_code" = "$expected_status" ]; then
        echo -e "${GREEN}✓ Status code correto ($status_code)${NC}"
    else
        echo -e "${RED}✗ Status code esperado: $expected_status, recebido: $status_code${NC}"
    fi
    
    echo -e "\n"
    sleep 1
}

# Teste 1: Criar usuário corretamente
test_request \
    "Teste 1: Criar usuário válido" \
    "POST" \
    "$API_URL/users" \
    '{"name": "João Silva", "email": "joao.silva@example.com", "age": 30}' \
    "201"

# Teste 2: Criar usuário com email inválido
test_request \
    "Teste 2: Criar usuário com email inválido" \
    "POST" \
    "$API_URL/users" \
    '{"name": "Maria Santos", "email": "email-invalido", "age": 25}' \
    "400"

# Teste 3: Criar usuário sem passar email (campo obrigatório)
test_request \
    "Teste 3: Criar usuário sem email (campo obrigatório)" \
    "POST" \
    "$API_URL/users" \
    '{"name": "Pedro Costa", "age": 28}' \
    "400"

# Teste 4: Criar usuário passando campo adicional (não permitido no POST)
test_request \
    "Teste 4: Criar usuário com campo adicional (não permitido)" \
    "POST" \
    "$API_URL/users" \
    '{"name": "Ana Lima", "email": "ana.lima@example.com", "age": 32, "extra_field": "não permitido"}' \
    "400"

# Teste 5: Criar usuário sem nome (campo obrigatório)
test_request \
    "Teste 5: Criar usuário sem nome (campo obrigatório)" \
    "POST" \
    "$API_URL/users" \
    '{"email": "sem.nome@example.com", "age": 22}' \
    "400"

# Teste 6: Criar usuário com nome muito curto (minLength: 2)
test_request \
    "Teste 6: Criar usuário com nome muito curto" \
    "POST" \
    "$API_URL/users" \
    '{"name": "A", "email": "nome.curto@example.com"}' \
    "400"

# Teste 7: Tentar criar usuário sem headers obrigatórios
echo -e "${YELLOW}--- Teste 7: Criar usuário sem headers obrigatórios ---${NC}"
echo "Request: POST $API_URL/users (sem headers de auth)"
echo "Data: {\"name\": \"Teste Header\", \"email\": \"teste.header@example.com\"}"

response=$(curl -s -w "\nHTTP_STATUS:%{http_code}" \
    -X "POST" \
    -H "Content-Type: application/json" \
    -d '{"name": "Teste Header", "email": "teste.header@example.com"}' \
    "$API_URL/users")

body=$(echo "$response" | sed '$d')
status_code=$(echo "$response" | grep "HTTP_STATUS:" | cut -d: -f2)

echo "Status Code: $status_code"
echo "Response Body:"
echo "$body" | jq . 2>/dev/null || echo "$body"

if [ "$status_code" = "400" ]; then
    echo -e "${GREEN}✓ Status code correto (400)${NC}"
else
    echo -e "${RED}✗ Status code esperado: 400, recebido: $status_code${NC}"
fi

echo -e "\n"

# Teste 8: Atualizar usuário com campo adicional (permitido no PUT)
test_request \
    "Teste 8: Atualizar usuário com campo adicional (permitido)" \
    "PUT" \
    "$API_URL/users/123" \
    '{"name": "João Atualizado", "custom_field": "isso é permitido no PUT"}' \
    "200"

echo -e "${BLUE}=== Fim dos Testes ===${NC}"

# Verificar se ainda há logs do servidor
echo -e "\n${BLUE}=== Logs do Servidor (últimas 20 linhas) ===${NC}"
if [ -f "server.log" ]; then
    tail -20 server.log
else
    echo "Arquivo de log não encontrado. Verificando processo..."
    ps aux | grep phlow | grep -v grep
fi
