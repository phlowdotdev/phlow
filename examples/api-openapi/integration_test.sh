#!/bin/bash

# =============================================================================
# SCRIPT DE TESTE DE INTEGRAÇÃO COMPLETO - API OPENAPI
# Valida todos os cenários possíveis de validação e funcionalidade
# =============================================================================

set -e  # Parar em caso de erro

# Configurações
BASE_URL="http://localhost:3000"
CONTENT_TYPE="Content-Type: application/json"
TIMEOUT=5

# Cores para output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Contadores para estatísticas
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Arrays para armazenar resultados detalhados
declare -a FAILED_TEST_DETAILS=()

# Função para imprimir cabeçalho colorido
print_header() {
    echo -e "\n${CYAN}============================================================"
    echo -e "$1"
    echo -e "============================================================${NC}\n"
}

# Função para imprimir subcabeçalho
print_subheader() {
    echo -e "\n${BLUE}--- $1 ---${NC}\n"
}

# Função para imprimir resultado do teste
print_result() {
    local test_name="$1"
    local expected_status="$2"
    local actual_status="$3"
    local success="$4"
    local response_body="$5"
    
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    if [ "$success" = "true" ]; then
        echo -e "${GREEN}✓ PASS${NC} - $test_name (Expected: $expected_status, Got: $actual_status)"
        PASSED_TESTS=$((PASSED_TESTS + 1))
        
        # Mostrar resposta se contém dados úteis
        if [ -n "$response_body" ] && [ "$response_body" != "{}" ] && [ ${#response_body} -lt 200 ]; then
            echo -e "  ${CYAN}Response:${NC} $response_body"
        fi
    else
        echo -e "${RED}✗ FAIL${NC} - $test_name (Expected: $expected_status, Got: $actual_status)"
        FAILED_TESTS=$((FAILED_TESTS + 1))
        
        # Armazenar detalhes da falha
        FAILED_TEST_DETAILS+=("$test_name: Expected $expected_status, Got $actual_status")
        
        if [ -n "$response_body" ]; then
            echo -e "  ${YELLOW}Response:${NC} $response_body"
        fi
    fi
    
    echo
}

# Função para fazer requisição HTTP e validar status
test_request() {
    local method="$1"
    local url="$2"
    local expected_status="$3"
    local test_name="$4"
    local data="$5"
    local extra_headers="$6"
    
    local headers_args=""
    local use_default_content_type=true
    
    if [ -n "$extra_headers" ]; then
        headers_args="$extra_headers"
        # Se extra_headers contém Content-Type, não usar o padrão
        if [[ "$extra_headers" == *"Content-Type"* ]]; then
            use_default_content_type=false
        fi
    fi
    
    if [ -n "$data" ]; then
        if [ "$use_default_content_type" = true ]; then
            response=$(timeout $TIMEOUT curl -s -w "\n%{http_code}" -X "$method" -H "$CONTENT_TYPE" $headers_args -d "$data" "$url" 2>/dev/null || echo -e "\nERROR")
        else
            response=$(timeout $TIMEOUT curl -s -w "\n%{http_code}" -X "$method" $headers_args -d "$data" "$url" 2>/dev/null || echo -e "\nERROR")
        fi
    else
        response=$(timeout $TIMEOUT curl -s -w "\n%{http_code}" -X "$method" $headers_args "$url" 2>/dev/null || echo -e "\nERROR")
    fi
    
    if [[ "$response" == *"ERROR" ]]; then
        print_result "$test_name" "$expected_status" "ERROR" "false" "Connection failed or timeout"
        return
    fi
    
    actual_status=$(echo "$response" | tail -n1)
    response_body=$(echo "$response" | head -n -1)
    
    # Verificar se o status obtido está na lista de status esperados (suporta múltiplos)
    if echo "$expected_status" | grep -q "\<$actual_status\>"; then
        print_result "$test_name" "$expected_status" "$actual_status" "true" "$response_body"
    else
        print_result "$test_name" "$expected_status" "$actual_status" "false" "$response_body"
    fi
}

# Função para aguardar entre testes
sleep_between_tests() {
    sleep 0.3
}

# Verificar se o servidor está rodando
check_server() {
    print_subheader "Verificando Conectividade do Servidor"
    
    if timeout $TIMEOUT curl -s -f "$BASE_URL/health" >/dev/null 2>&1; then
        echo -e "${GREEN}✓ Servidor está rodando e respondendo em $BASE_URL${NC}\n"
    else
        echo -e "${RED}✗ ERRO: Servidor não está respondendo em $BASE_URL${NC}"
        echo -e "${YELLOW}Certifique-se de que o servidor está rodando com:${NC}"
        echo -e "${CYAN}  PHLOW_LOG=debug phlow examples/api-openapi${NC}\n"
        exit 1
    fi
}

# Função para limpar dados de teste (usuários criados)
cleanup_test_data() {
    echo -e "${YELLOW}Limpando dados de teste...${NC}"
    # Em um cenário real, aqui limparíamos o cache ou banco de dados
    # Para este exemplo, o cache é limpo reiniciando o servidor
    echo -e "${CYAN}Dados de teste serão limpos automaticamente pelo cache temporário${NC}\n"
}

print_header "INICIANDO BATERIA COMPLETA DE TESTES DE INTEGRAÇÃO"
echo -e "${MAGENTA}Data/Hora: $(date)${NC}"
echo -e "${MAGENTA}Testando contra: $BASE_URL${NC}"

check_server

# =============================================================================
# A. TESTES DE CONECTIVIDADE E ENDPOINTS ESPECIAIS
# =============================================================================

print_header "A. TESTES DE CONECTIVIDADE E ENDPOINTS ESPECIAIS"

print_subheader "A.1 - Endpoints de Sistema"

test_request "GET" "$BASE_URL/health" "200" \
    "Health check endpoint"

sleep_between_tests

test_request "GET" "$BASE_URL/openapi.json" "200" \
    "OpenAPI specification endpoint"

sleep_between_tests

# =============================================================================
# B. TESTES POST /users (CRIAÇÃO DE USUÁRIOS)
# =============================================================================

print_header "B. TESTES POST /users - CRIAÇÃO DE USUÁRIOS"

print_subheader "B.1 - Casos Válidos"

test_request "POST" "$BASE_URL/users" "201" \
    "Criar usuário com dados completos válidos" \
    '{"name":"João Silva","email":"joao@exemplo.com","age":30}'

sleep_between_tests

test_request "POST" "$BASE_URL/users" "201" \
    "Criar usuário com dados mínimos obrigatórios" \
    '{"name":"Maria Santos","email":"maria@exemplo.com"}'

sleep_between_tests

test_request "POST" "$BASE_URL/users" "400" \
    "Criar usuário com propriedades adicionais (deve rejeitar)" \
    '{"name":"Pedro Oliveira","email":"pedro@exemplo.com","age":28,"city":"São Paulo","country":"Brasil"}'

sleep_between_tests

print_subheader "B.2 - Validação de Campos Obrigatórios"

test_request "POST" "$BASE_URL/users" "400" \
    "Sem campo name (obrigatório)" \
    '{"email":"teste@exemplo.com","age":25}'

sleep_between_tests

test_request "POST" "$BASE_URL/users" "400" \
    "Sem campo email (obrigatório)" \
    '{"name":"Teste User","age":25}'

sleep_between_tests

test_request "POST" "$BASE_URL/users" "400" \
    "Sem ambos os campos obrigatórios" \
    '{"age":25}'

sleep_between_tests

test_request "POST" "$BASE_URL/users" "400" \
    "Body vazio" \
    '{}'

sleep_between_tests

test_request "POST" "$BASE_URL/users" "400" \
    "Sem body" \
    ''

sleep_between_tests

print_subheader "B.3 - Validação de Nome"

test_request "POST" "$BASE_URL/users" "400" \
    "Nome muito curto (1 caractere)" \
    '{"name":"A","email":"teste@exemplo.com"}'

sleep_between_tests

test_request "POST" "$BASE_URL/users" "400" \
    "Nome muito longo (>50 caracteres)" \
    '{"name":"'"$(printf '%0.s' {1..55} | tr '0' 'A')"'","email":"teste@exemplo.com"}'

sleep_between_tests

test_request "POST" "$BASE_URL/users" "400" \
    "Nome com números" \
    '{"name":"João123","email":"teste@exemplo.com"}'

sleep_between_tests

test_request "POST" "$BASE_URL/users" "400" \
    "Nome com caracteres especiais inválidos" \
    '{"name":"João@Silva#","email":"teste@exemplo.com"}'

sleep_between_tests

test_request "POST" "$BASE_URL/users" "201" \
    "Nome válido com espaços" \
    '{"name":"João Silva","email":"teste1@exemplo.com"}'

sleep_between_tests

print_subheader "B.4 - Validação de Email"

test_request "POST" "$BASE_URL/users" "400" \
    "Email sem @" \
    '{"name":"Teste","email":"email-sem-arroba"}'

sleep_between_tests

test_request "POST" "$BASE_URL/users" "400" \
    "Email sem domínio" \
    '{"name":"Teste","email":"usuario@"}'

sleep_between_tests

test_request "POST" "$BASE_URL/users" "400" \
    "Email sem TLD" \
    '{"name":"Teste","email":"usuario@dominio"}'

sleep_between_tests

test_request "POST" "$BASE_URL/users" "201" \
    "Email com formato válido básico" \
    '{"name":"Teste","email":"usuario@dominio.com"}'

sleep_between_tests

test_request "POST" "$BASE_URL/users" "201" \
    "Email com subdomínio" \
    '{"name":"Teste","email":"usuario@sub.dominio.com.br"}'

sleep_between_tests

print_subheader "B.5 - Validação de Idade"

test_request "POST" "$BASE_URL/users" "400" \
    "Idade negativa" \
    '{"name":"Teste","email":"idade1@exemplo.com","age":-5}'

sleep_between_tests

test_request "POST" "$BASE_URL/users" "400" \
    "Idade muito alta (>120)" \
    '{"name":"Teste","email":"idade2@exemplo.com","age":150}'

sleep_between_tests

test_request "POST" "$BASE_URL/users" "400" \
    "Idade como string" \
    '{"name":"Teste","email":"idade3@exemplo.com","age":"25"}'

sleep_between_tests

test_request "POST" "$BASE_URL/users" "201" \
    "Idade válida" \
    '{"name":"Teste","email":"idade4@exemplo.com","age":25}'

sleep_between_tests

print_subheader "B.6 - Validação de Telefone"

test_request "POST" "$BASE_URL/users" "400" \
    "Telefone com formato inválido" \
    '{"name":"Teste","email":"phone1@exemplo.com","phone":"123abc"}'

sleep_between_tests

test_request "POST" "$BASE_URL/users" "201" \
    "Telefone com formato válido internacional" \
    '{"name":"Teste","email":"phone2@exemplo.com","phone":"+5511999999999"}'

sleep_between_tests

test_request "POST" "$BASE_URL/users" "201" \
    "Telefone com formato válido nacional" \
    '{"name":"Teste","email":"phone3@exemplo.com","phone":"11999999999"}'

sleep_between_tests

# =============================================================================
# C. TESTES GET /users (LISTAGEM)
# =============================================================================

print_header "C. TESTES GET /users - LISTAGEM DE USUÁRIOS"

test_request "GET" "$BASE_URL/users" "200" \
    "Listar todos os usuários"

sleep_between_tests

# =============================================================================
# D. TESTES GET /users/{userId} (BUSCA INDIVIDUAL)
# =============================================================================

print_header "D. TESTES GET /users/{userId} - BUSCA INDIVIDUAL"

test_request "GET" "$BASE_URL/users/joao@exemplo.com" "200" \
    "Buscar usuário existente (usando email como ID)"

sleep_between_tests

test_request "GET" "$BASE_URL/users/usuario-inexistente" "404" \
    "Buscar usuário inexistente"

sleep_between_tests

test_request "GET" "$BASE_URL/users/123" "404" \
    "Buscar com ID numérico (inexistente)"

sleep_between_tests

# =============================================================================
# E. TESTES PUT /users/{userId} (ATUALIZAÇÃO)
# =============================================================================

print_header "E. TESTES PUT /users/{userId} - ATUALIZAÇÃO DE USUÁRIOS"

test_request "PUT" "$BASE_URL/users/joao@exemplo.com" "200" \
    "Atualizar usuário existente" \
    '{"name":"João Silva Atualizado","age":31}'

sleep_between_tests

test_request "PUT" "$BASE_URL/users/joao@exemplo.com" "200" \
    "Atualização parcial (apenas nome)" \
    '{"name":"Novo Nome"}'

sleep_between_tests

test_request "PUT" "$BASE_URL/users/joao@exemplo.com" "200" \
    "Atualização parcial (apenas idade)" \
    '{"age":32}'

sleep_between_tests

test_request "PUT" "$BASE_URL/users/usuario-inexistente" "404" \
    "Atualizar usuário inexistente" \
    '{"name":"Teste","age":25}'

sleep_between_tests

test_request "PUT" "$BASE_URL/users/joao@exemplo.com" "400" \
    "Atualização com dados inválidos" \
    '{"name":"João123"}'

sleep_between_tests

# =============================================================================
# F. TESTES DELETE /users/{userId} (REMOÇÃO)
# =============================================================================

print_header "F. TESTES DELETE /users/{userId} - REMOÇÃO DE USUÁRIOS"

test_request "DELETE" "$BASE_URL/users/maria@exemplo.com" "204" \
    "Deletar usuário existente"

sleep_between_tests

test_request "DELETE" "$BASE_URL/users/usuario-inexistente" "404" \
    "Deletar usuário inexistente"

sleep_between_tests

test_request "DELETE" "$BASE_URL/users/abc123" "404" \
    "Deletar com ID inválido"

sleep_between_tests

# =============================================================================
# G. TESTES DE MÉTODOS NÃO PERMITIDOS
# =============================================================================

print_header "G. TESTES DE MÉTODOS NÃO PERMITIDOS"

test_request "PATCH" "$BASE_URL/users" "405" \
    "PATCH não permitido em /users"

sleep_between_tests

test_request "DELETE" "$BASE_URL/users" "405" \
    "DELETE não permitido em /users"

sleep_between_tests

test_request "POST" "$BASE_URL/users/123" "405" \
    "POST não permitido em /users/{userId}" \
    '{"name":"Teste"}'

sleep_between_tests

test_request "PATCH" "$BASE_URL/users/123" "405" \
    "PATCH não permitido em /users/{userId}" \
    '{"name":"Teste"}'

sleep_between_tests

# =============================================================================
# H. TESTES DE ROTAS INEXISTENTES
# =============================================================================

print_header "H. TESTES DE ROTAS INEXISTENTES"

test_request "GET" "$BASE_URL/rota-inexistente" "404" \
    "Rota completamente inexistente"

sleep_between_tests

test_request "GET" "$BASE_URL/users/123/profile" "404" \
    "Subrota inexistente"

sleep_between_tests

test_request "GET" "$BASE_URL/api/v1/users" "404" \
    "Rota com path prefix inexistente"

sleep_between_tests

# =============================================================================
# I. TESTES DE FORMATO E HEADERS
# =============================================================================

print_header "I. TESTES DE FORMATO E HEADERS"

test_request "POST" "$BASE_URL/users" "400" \
    "JSON malformado" \
    '{"name":"Teste",'

sleep_between_tests

test_request "POST" "$BASE_URL/users" "400" \
    "Content-Type incorreto" \
    '{"name":"Teste","email":"teste@exemplo.com"}' \
    '-H "Content-Type: text/plain"'

sleep_between_tests

test_request "POST" "$BASE_URL/users" "400" \
    "Dados não-JSON com Content-Type JSON" \
    'nome=Teste&email=teste@exemplo.com'

sleep_between_tests

# =============================================================================
# J. TESTES DE EDGE CASES
# =============================================================================

print_header "J. TESTES DE EDGE CASES E CENÁRIOS ESPECIAIS"

test_request "POST" "$BASE_URL/users" "400" \
    "Email com espaços" \
    '{"name":"Teste","email":"teste @exemplo.com"}'

sleep_between_tests

test_request "POST" "$BASE_URL/users" "400" \
    "Nome com apenas espaços" \
    '{"name":"   ","email":"teste@exemplo.com"}'

sleep_between_tests

test_request "POST" "$BASE_URL/users" "400" \
    "Idade como decimal" \
    '{"name":"Teste","email":"decimal@exemplo.com","age":25.5}'

sleep_between_tests

test_request "POST" "$BASE_URL/users" "201" \
    "Usuário com idade zero (limite inferior)" \
    '{"name":"Baby","email":"baby@exemplo.com","age":0}'

sleep_between_tests

test_request "POST" "$BASE_URL/users" "201" \
    "Usuário com idade 120 (limite superior)" \
    '{"name":"Centenário","email":"old@exemplo.com","age":120}'

sleep_between_tests

test_request "POST" "$BASE_URL/users" "400" \
    "Múltiplos campos inválidos simultaneamente" \
    '{"name":"123","email":"email-inválido","age":-10}'

sleep_between_tests

# =============================================================================
# ESTATÍSTICAS FINAIS E RELATÓRIO
# =============================================================================

print_header "RELATÓRIO FINAL DE TESTES DE INTEGRAÇÃO"

echo -e "${CYAN}Estatísticas Gerais:${NC}"
echo -e "  ${BLUE}Total de testes executados:${NC} $TOTAL_TESTS"
echo -e "  ${GREEN}Testes aprovados:${NC} $PASSED_TESTS"
echo -e "  ${RED}Testes falharam:${NC} $FAILED_TESTS"

if [ $TOTAL_TESTS -gt 0 ]; then
    PASS_RATE=$((PASSED_TESTS * 100 / TOTAL_TESTS))
    echo -e "  ${MAGENTA}Taxa de sucesso:${NC} ${PASS_RATE}%"
fi

echo

if [ $FAILED_TESTS -eq 0 ]; then
    echo -e "${GREEN}🎉 TODOS OS TESTES PASSARAM!${NC}"
    echo -e "${GREEN}✅ A API está funcionando corretamente em todos os cenários testados.${NC}"
    exit_code=0
else
    echo -e "${RED}❌ ALGUNS TESTES FALHARAM${NC}"
    echo -e "${YELLOW}Detalhes das falhas:${NC}"
    
    for detail in "${FAILED_TEST_DETAILS[@]}"; do
        echo -e "  ${RED}•${NC} $detail"
    done
    
    echo
    echo -e "${YELLOW}Recomendações:${NC}"
    echo -e "  1. Verificar logs do servidor para mais detalhes"
    echo -e "  2. Confirmar se todas as validações OpenAPI estão implementadas"
    echo -e "  3. Testar individualmente os casos que falharam"
    
    exit_code=1
fi

echo
echo -e "${CYAN}Teste concluído em: $(date)${NC}"

cleanup_test_data

exit $exit_code
