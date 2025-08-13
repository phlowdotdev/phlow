#!/bin/bash

# =============================================================================
# SCRIPT AUXILIAR - EXECUTAR SERVIDOR E TESTES DE INTEGRAÇÃO
# =============================================================================

set -e

# Cores
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo -e "${BLUE}🚀 INICIANDO EXECUÇÃO COMPLETA DOS TESTES DE INTEGRAÇÃO${NC}\n"

echo -e "${YELLOW}Diretório do projeto: $PROJECT_ROOT${NC}"
echo -e "${YELLOW}Diretório dos exemplos: $SCRIPT_DIR${NC}\n"

# Função para cleanup ao sair
cleanup() {
    echo -e "\n${YELLOW}🧹 Limpando processos...${NC}"
    if [ -n "$SERVER_PID" ]; then
        echo -e "${YELLOW}Parando servidor (PID: $SERVER_PID)...${NC}"
        kill $SERVER_PID 2>/dev/null || true
        wait $SERVER_PID 2>/dev/null || true
    fi
    
    # Matar qualquer processo phlow que ainda esteja rodando
    pkill -f "phlow examples/api-openapi" 2>/dev/null || true
    
    echo -e "${GREEN}✅ Cleanup concluído${NC}"
}

# Registrar função de cleanup para ser chamada ao sair
trap cleanup EXIT

cd "$PROJECT_ROOT"

# 1. Buildar o módulo http_server
echo -e "${BLUE}📦 Buildando módulo http_server...${NC}"
if cargo make local http_server; then
    echo -e "${GREEN}✅ Módulo buildado com sucesso${NC}\n"
else
    echo -e "${RED}❌ Erro ao buildar módulo${NC}"
    exit 1
fi

# 2. Matar processos anteriores
echo -e "${YELLOW}🔪 Parando processos anteriores do servidor...${NC}"
pkill -f "phlow examples/api-openapi" 2>/dev/null || true
sleep 2

# 3. Iniciar o servidor
echo -e "${BLUE}🌐 Iniciando servidor Phlow...${NC}"
PHLOW_LOG=debug phlow examples/api-openapi > server_test.log 2>&1 &
SERVER_PID=$!

echo -e "${YELLOW}Servidor iniciado com PID: $SERVER_PID${NC}"
echo -e "${YELLOW}Aguardando servidor inicializar...${NC}"

# 4. Aguardar servidor estar pronto
RETRY_COUNT=0
MAX_RETRIES=15

while [ $RETRY_COUNT -lt $MAX_RETRIES ]; do
    if curl -s -f http://localhost:3000/health > /dev/null 2>&1; then
        echo -e "${GREEN}✅ Servidor está respondendo!${NC}\n"
        break
    fi
    
    RETRY_COUNT=$((RETRY_COUNT + 1))
    echo -e "${YELLOW}Tentativa $RETRY_COUNT/$MAX_RETRIES...${NC}"
    sleep 1
done

if [ $RETRY_COUNT -eq $MAX_RETRIES ]; then
    echo -e "${RED}❌ Servidor não respondeu após $MAX_RETRIES tentativas${NC}"
    echo -e "${YELLOW}Logs do servidor:${NC}"
    tail -20 server_test.log
    exit 1
fi

# 5. Executar testes de integração
echo -e "${BLUE}🧪 Executando testes de integração...${NC}\n"

if "$SCRIPT_DIR/integration_test.sh"; then
    echo -e "\n${GREEN}🎉 TODOS OS TESTES PASSARAM!${NC}"
    TEST_RESULT=0
else
    echo -e "\n${RED}❌ ALGUNS TESTES FALHARAM${NC}"
    TEST_RESULT=1
fi

# 6. Mostrar logs do servidor se houve falha
if [ $TEST_RESULT -ne 0 ]; then
    echo -e "\n${YELLOW}📋 Últimas linhas do log do servidor:${NC}"
    tail -50 server_test.log
fi

echo -e "\n${BLUE}📊 RESUMO DA EXECUÇÃO${NC}"
echo -e "${CYAN}• Servidor PID: $SERVER_PID${NC}"
echo -e "${CYAN}• Log do servidor: server_test.log${NC}"
echo -e "${CYAN}• Script de teste: $SCRIPT_DIR/integration_test.sh${NC}"

if [ $TEST_RESULT -eq 0 ]; then
    echo -e "${GREEN}• Resultado: ✅ SUCESSO${NC}"
else
    echo -e "${RED}• Resultado: ❌ FALHA${NC}"
fi

echo -e "\n${BLUE}Execução concluída em: $(date)${NC}\n"

exit $TEST_RESULT
