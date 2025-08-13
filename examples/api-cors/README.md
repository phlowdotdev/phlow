# 🌍 Modern API with CORS Support

Este exemplo demonstra uma API REST completa com suporte a CORS (Cross-Origin Resource Sharing) usando o módulo HTTP Server do Phlow.

## 🚀 Funcionalidades

### ✨ **API Features**
- **RESTful API** completa com recursos Users e Posts
- **CORS configurado** para desenvolvimento e produção
- **Versionamento de API** (v1)
- **Paginação** para listagens
- **Validação de dados** de entrada
- **Headers personalizados** e metadata de API
- **Logging automático** de requisições
- **Tratamento de erros** estruturado

### 🌐 **CORS Configuration**
- **Origins permitidas**: Localhost (React, Angular, Vite) + domínio de produção
- **Métodos HTTP**: GET, POST, PUT, PATCH, DELETE, OPTIONS
- **Headers suportados**: Content-Type, Authorization, X-API-Key, etc.
- **Credentials**: Habilitado para cookies e auth headers
- **Preflight cache**: 2 horas

## 📋 Endpoints da API

### 🏠 **Root Endpoints**
```
GET  /              # Documentação da API
GET  /health        # Health check automático
GET  /api/          # Informações da API
```

### 👥 **Users API**
```
GET    /api/v1/users           # Lista todos os usuários
GET    /api/v1/users?page=1    # Lista paginada
GET    /api/v1/users/:id       # Obtém usuário específico
POST   /api/v1/users           # Cria novo usuário
PUT    /api/v1/users/:id       # Atualiza usuário
DELETE /api/v1/users/:id       # Remove usuário
```

### 📝 **Posts API**
```
GET  /api/v1/posts      # Lista todos os posts
GET  /api/v1/posts/:id  # Obtém post específico  
POST /api/v1/posts      # Cria novo post
```

## 🔧 Como usar

### 1. **Iniciar a API**
```bash
# No diretório do Phlow
phlow examples/api-cors/api-cors.phlow
```

A API estará disponível em: `http://localhost:8080`

### 2. **Testar Endpoints Básicos**

#### Health Check
```bash
curl http://localhost:8080/health
```

#### Documentação da API
```bash
curl http://localhost:8080/ | jq
```

#### Info da API
```bash
curl http://localhost:8080/api/ | jq
```

### 3. **Testar Users API**

#### Listar usuários
```bash
curl http://localhost:8080/api/v1/users | jq
```

#### Listar com paginação
```bash
curl "http://localhost:8080/api/v1/users?page=2&limit=5" | jq
```

#### Obter usuário específico
```bash
curl http://localhost:8080/api/v1/users/1 | jq
```

#### Criar novo usuário
```bash
curl -X POST http://localhost:8080/api/v1/users \
  -H "Content-Type: application/json" \
  -d '{"name": "John Doe", "email": "john@example.com"}' | jq
```

#### Atualizar usuário
```bash
curl -X PUT http://localhost:8080/api/v1/users/1 \
  -H "Content-Type: application/json" \
  -d '{"name": "John Updated", "email": "john.updated@example.com"}' | jq
```

#### Remover usuário
```bash
curl -X DELETE http://localhost:8080/api/v1/users/1 | jq
```

### 4. **Testar Posts API**

#### Listar posts
```bash
curl http://localhost:8080/api/v1/posts | jq
```

#### Obter post específico
```bash
curl http://localhost:8080/api/v1/posts/1 | jq
```

#### Criar novo post
```bash
curl -X POST http://localhost:8080/api/v1/posts \
  -H "Content-Type: application/json" \
  -d '{
    "title": "Meu Novo Post",
    "content": "Este é o conteúdo do meu post...",
    "author_id": 1,
    "tags": ["tutorial", "api"],
    "published": true
  }' | jq
```

## 🌐 Testando CORS

### 1. **Preflight Request (OPTIONS)**
```bash
curl -X OPTIONS http://localhost:8080/api/v1/users \
  -H "Origin: http://localhost:3000" \
  -H "Access-Control-Request-Method: POST" \
  -H "Access-Control-Request-Headers: Content-Type, Authorization" \
  -v
```

**Resposta esperada:**
```
HTTP/1.1 200 OK
Access-Control-Allow-Origin: http://localhost:3000
Access-Control-Allow-Methods: GET, POST, PUT, PATCH, DELETE, OPTIONS
Access-Control-Allow-Headers: Content-Type, Authorization, X-Requested-With, X-API-Key, X-Client-Version, Accept
Access-Control-Allow-Credentials: true
Access-Control-Max-Age: 7200
```

### 2. **Request com Origin Permitido**
```bash
curl -X GET http://localhost:8080/api/v1/users \
  -H "Origin: http://localhost:3000" \
  -v
```

**Resposta esperada:**
```
HTTP/1.1 200 OK
Access-Control-Allow-Origin: http://localhost:3000
Access-Control-Allow-Credentials: true
Content-Type: application/json
```

### 3. **Request com Origin Não Permitido**
```bash
curl -X GET http://localhost:8080/api/v1/users \
  -H "Origin: https://malicious-site.com" \
  -v
```

**Comportamento esperado:** Sem headers CORS na resposta (bloqueado pelo browser)

## 🧪 Testando no Frontend

### **JavaScript/Fetch**
```javascript
// Exemplo para testar CORS em uma aplicação frontend
async function testCorsApi() {
  try {
    // Listar usuários
    const response = await fetch('http://localhost:8080/api/v1/users', {
      method: 'GET',
      credentials: 'include', // Inclui cookies
      headers: {
        'Content-Type': 'application/json',
        'X-Client-Version': '1.0.0'
      }
    });
    
    const users = await response.json();
    console.log('Users:', users);
    
    // Criar usuário
    const createResponse = await fetch('http://localhost:8080/api/v1/users', {
      method: 'POST',
      credentials: 'include',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': 'Bearer abc123'
      },
      body: JSON.stringify({
        name: 'Frontend User',
        email: 'frontend@example.com'
      })
    });
    
    const newUser = await createResponse.json();
    console.log('New User:', newUser);
    
  } catch (error) {
    console.error('CORS Error:', error);
  }
}

// Executar teste
testCorsApi();
```

### **Axios (React/Vue/Angular)**
```javascript
import axios from 'axios';

// Configuração global do Axios
axios.defaults.baseURL = 'http://localhost:8080';
axios.defaults.withCredentials = true; // Para CORS com credentials

// Interceptor para adicionar headers
axios.interceptors.request.use(config => {
  config.headers['X-Client-Version'] = '1.0.0';
  return config;
});

// Exemplo de uso
async function fetchUsers() {
  try {
    const response = await axios.get('/api/v1/users');
    return response.data;
  } catch (error) {
    console.error('API Error:', error);
  }
}
```

## 📝 **Estrutura de Dados**

### **User Object**
```json
{
  "id": 123,
  "name": "John Doe",
  "email": "john@example.com",
  "avatar": "https://api.dicebear.com/7.x/avataaars/svg?seed=john",
  "created_at": "2024-01-01T00:00:00Z",
  "updated_at": "2024-01-01T00:00:00Z"
}
```

### **Post Object**
```json
{
  "id": 456,
  "title": "Meu Post Incrível",
  "content": "Conteúdo completo do post...",
  "author_id": 123,
  "author_name": "John Doe",
  "tags": ["tutorial", "api", "cors"],
  "published": true,
  "created_at": "2024-01-01T00:00:00Z",
  "updated_at": "2024-01-01T00:00:00Z"
}
```

### **Error Response**
```json
{
  "error": "Bad Request",
  "message": "Email is required",
  "field": "email"
}
```

### **Paginated Response**
```json
{
  "users": [/* user objects */],
  "pagination": {
    "page": 1,
    "limit": 10,
    "total": 1000,
    "pages": 100
  }
}
```

## 🔍 **Headers de Resposta**

Todas as respostas incluem os seguintes headers:

### **CORS Headers**
- `Access-Control-Allow-Origin`: Origin específico ou "*"
- `Access-Control-Allow-Methods`: Métodos HTTP permitidos
- `Access-Control-Allow-Headers`: Headers permitidos
- `Access-Control-Allow-Credentials`: true/false
- `Access-Control-Max-Age`: Cache preflight

### **API Headers**
- `Content-Type`: application/json
- `X-API-Version`: Versão da API (v1)
- `X-Total-Count`: Total de itens (listas)
- `X-Page`: Página atual (paginação)
- `X-Per-Page`: Itens por página
- `Location`: URL do recurso criado (POST)

## 🛠️ **Personalização**

### **Modificar Origins Permitidos**
Edite o arquivo `api-cors.phlow`:

```yaml
cors:
  origins:
    - "http://localhost:3000"      # Seu frontend local
    - "https://meudominio.com"     # Sua aplicação em produção
    - "https://app.exemplo.com"    # Subdomínio específico
```

### **Adicionar Novos Endpoints**
Adicione novos recursos seguindo o padrão:

```yaml
steps:
  - name: "route_handler"
    condition:
      left: "{{ $api_router.resource }}"
      operator: "equals"
      right: "meu-novo-recurso"
    then:
      # Implementar handlers GET, POST, etc.
```

### **Customizar Headers CORS**
```yaml
cors:
  headers:
    - "Content-Type"
    - "Authorization"
    - "X-API-Key"
    - "X-Custom-Header"     # Seu header personalizado
```

## 📊 **Observabilidade**

A API inclui logging automático via módulo `log`:
- **Requisições**: Método, path, origin
- **CORS**: Headers de preflight e validação
- **Erros**: Requests inválidos e origins negados

### **Exemplo de Log**
```
[INFO] API Request: GET /api/v1/users from http://localhost:3000
[INFO] API Request: POST /api/v1/users from https://myapp.com
[INFO] API Request: OPTIONS /api/v1/posts from http://localhost:3000
```

## 🔐 **Considerações de Segurança**

### **Para Desenvolvimento**
```yaml
cors:
  origins: ["*"]          # Aceita qualquer origin
  credentials: false      # Mais seguro para dev
```

### **Para Produção**
```yaml
cors:
  origins: 
    - "https://meuapp.com"     # Apenas domínios específicos
    - "https://www.meuapp.com" # Com e sem www
  credentials: true            # Para autenticação
  max_age: 86400              # Cache longo para performance
```

## 🚀 **Próximos Passos**

1. **Autenticação**: Adicionar middleware de JWT
2. **Rate Limiting**: Implementar limites de requisição
3. **Validação**: Schema validation com OpenAPI
4. **Database**: Integrar com banco de dados real
5. **Cache**: Adicionar cache de respostas
6. **Websockets**: Suporte a real-time

---

**🎯 Este exemplo demonstra uma implementação completa e pronta para produção de uma API REST com CORS usando Phlow!**
