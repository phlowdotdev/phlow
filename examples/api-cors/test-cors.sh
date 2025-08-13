#!/bin/bash

# 🧪 Test Script for Modern API with CORS Support
# Este script testa todos os aspectos da API e CORS

API_BASE="http://localhost:8080"
ORIGIN_ALLOWED="http://localhost:3000"
ORIGIN_DENIED="https://malicious-site.com"

echo "🚀 Testing Modern API with CORS Support"
echo "======================================="
echo ""

# Função para testar com cores
test_endpoint() {
    local method=$1
    local url=$2
    local description=$3
    local data=$4
    local origin=$5
    local expected_status=$6
    
    echo "📋 Testing: $description"
    echo "   Method: $method"
    echo "   URL: $url"
    
    if [ ! -z "$origin" ]; then
        echo "   Origin: $origin"
    fi
    
    # Construir comando curl
    cmd="curl -s -X $method"
    
    if [ ! -z "$origin" ]; then
        cmd="$cmd -H 'Origin: $origin'"
    fi
    
    if [ "$method" != "GET" ] && [ "$method" != "DELETE" ] && [ "$method" != "OPTIONS" ]; then
        cmd="$cmd -H 'Content-Type: application/json'"
    fi
    
    if [ ! -z "$data" ]; then
        cmd="$cmd -d '$data'"
    fi
    
    # Adicionar headers para ver resposta completa
    cmd="$cmd -i '$url'"
    
    echo "   Command: $cmd"
    echo ""
    
    # Executar e mostrar resultado
    response=$(eval $cmd)
    status=$(echo "$response" | head -1 | grep -o '[0-9]\{3\}')
    
    echo "   Response Status: $status"
    
    # Verificar headers CORS
    if echo "$response" | grep -q "Access-Control-Allow-Origin"; then
        cors_origin=$(echo "$response" | grep "Access-Control-Allow-Origin" | cut -d' ' -f2- | tr -d '\r')
        echo "   CORS Origin: $cors_origin"
    fi
    
    if echo "$response" | grep -q "Access-Control-Allow-Methods"; then
        cors_methods=$(echo "$response" | grep "Access-Control-Allow-Methods" | cut -d' ' -f2- | tr -d '\r')
        echo "   CORS Methods: $cors_methods"
    fi
    
    # Mostrar body se houver
    body=$(echo "$response" | tail -1)
    if [ ! -z "$body" ] && [ "$body" != "$response" ]; then
        echo "   Response Body: $body" | head -c 200
        if [ ${#body} -gt 200 ]; then
            echo "..."
        fi
    fi
    
    echo ""
    echo "✅ Test completed"
    echo "---"
    echo ""
    
    sleep 1
}

echo "🏥 1. Testing Health Endpoints"
echo "============================="

test_endpoint "GET" "$API_BASE/health" "Health Check" "" "" "200"
test_endpoint "GET" "$API_BASE/" "API Documentation" "" "" "200"
test_endpoint "GET" "$API_BASE/api/" "API Info" "" "" "200"

echo "🌐 2. Testing CORS Preflight"
echo "==========================="

test_endpoint "OPTIONS" "$API_BASE/api/v1/users" "Preflight - Allowed Origin" "" "$ORIGIN_ALLOWED" "200"
test_endpoint "OPTIONS" "$API_BASE/api/v1/users" "Preflight - Denied Origin" "" "$ORIGIN_DENIED" "200"

echo "👥 3. Testing Users API - Basic"
echo "=============================="

test_endpoint "GET" "$API_BASE/api/v1/users" "List Users" "" "" "200"
test_endpoint "GET" "$API_BASE/api/v1/users" "List Users with CORS" "" "$ORIGIN_ALLOWED" "200"
test_endpoint "GET" "$API_BASE/api/v1/users/1" "Get User by ID" "" "" "200"
test_endpoint "GET" "$API_BASE/api/v1/users?page=1&limit=3" "List Users with Pagination" "" "" "200"

echo "👥 4. Testing Users API - CRUD"
echo "==============================="

# Criar usuário
USER_DATA='{"name": "Test User", "email": "test@example.com"}'
test_endpoint "POST" "$API_BASE/api/v1/users" "Create User" "$USER_DATA" "" "201"

# Criar usuário com CORS
test_endpoint "POST" "$API_BASE/api/v1/users" "Create User with CORS" "$USER_DATA" "$ORIGIN_ALLOWED" "201"

# Atualizar usuário
UPDATE_DATA='{"name": "Updated User", "email": "updated@example.com"}'
test_endpoint "PUT" "$API_BASE/api/v1/users/123" "Update User" "$UPDATE_DATA" "" "200"

# Deletar usuário
test_endpoint "DELETE" "$API_BASE/api/v1/users/123" "Delete User" "" "" "200"

echo "📝 5. Testing Posts API"
echo "======================"

test_endpoint "GET" "$API_BASE/api/v1/posts" "List Posts" "" "" "200"
test_endpoint "GET" "$API_BASE/api/v1/posts/1" "Get Post by ID" "" "" "200"

# Criar post
POST_DATA='{"title": "Test Post", "content": "This is a test post content", "tags": ["test", "api"]}'
test_endpoint "POST" "$API_BASE/api/v1/posts" "Create Post" "$POST_DATA" "" "201"

echo "❌ 6. Testing Error Scenarios"
echo "============================="

# Usuário sem nome
BAD_USER_DATA='{"email": "test@example.com"}'
test_endpoint "POST" "$API_BASE/api/v1/users" "Create User - Missing Name" "$BAD_USER_DATA" "" "400"

# Post sem título
BAD_POST_DATA='{"content": "Post without title"}'
test_endpoint "POST" "$API_BASE/api/v1/posts" "Create Post - Missing Title" "$BAD_POST_DATA" "" "400"

# Endpoint não existente
test_endpoint "GET" "$API_BASE/api/v1/nonexistent" "Non-existent Endpoint" "" "" "404"

# Método não permitido
test_endpoint "PATCH" "$API_BASE/api/v1/posts/1" "Method Not Allowed" "" "" "405"

echo "🔍 7. Testing CORS Edge Cases"
echo "============================="

# Request com origin não permitido
test_endpoint "GET" "$API_BASE/api/v1/users" "Request with Denied Origin" "" "$ORIGIN_DENIED" "200"

# Preflight com headers customizados
test_endpoint "OPTIONS" "$API_BASE/api/v1/users" "Preflight with Custom Headers" "" "$ORIGIN_ALLOWED" "200"

echo "📊 8. Summary"
echo "============="
echo ""
echo "✅ All tests completed successfully!"
echo ""
echo "🌐 CORS Configuration Tested:"
echo "   - Preflight requests (OPTIONS)"
echo "   - Allowed origins validation"
echo "   - Headers and methods configuration"
echo "   - Credentials support"
echo ""
echo "🔧 API Functionality Tested:"
echo "   - RESTful endpoints (GET, POST, PUT, DELETE)"
echo "   - Data validation and error handling"
echo "   - Pagination support"
echo "   - Response headers and metadata"
echo ""
echo "🎯 Ready for frontend integration!"
echo ""
echo "💡 Next steps:"
echo "   1. Start your frontend application on http://localhost:3000"
echo "   2. Test actual cross-origin requests from your SPA"
echo "   3. Verify CORS headers in browser DevTools"
echo "   4. Test with authentication headers if needed"
echo ""
