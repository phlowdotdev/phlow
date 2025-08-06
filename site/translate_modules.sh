#!/bin/bash

# Script to translate Portuguese module documentation to English

echo "Starting translation of module documentation from Portuguese to English..."

# Define base directory
DOCS_DIR="/home/assis/projects/lowcarboncode/phlow/site/docs/modules"

# Function to translate common Portuguese terms to English
translate_file() {
    local file="$1"
    echo "Translating $file..."
    
    # Create backup
    cp "$file" "$file.backup"
    
    # Translate Portuguese sections to English using sed
    sed -i \
        -e 's/## 📋 Configuração/## 📋 Configuration/g' \
        -e 's/### Configuração Básica/### Basic Configuration/g' \
        -e 's/### Configuração Avançada/### Advanced Configuration/g' \
        -e 's/### Configuração do Módulo (with)/### Module Configuration (with)/g' \
        -e 's/### Configuração com/### Configuration with/g' \
        -e 's/### Configuração para Produção/### Production Configuration/g' \
        -e 's/### Configuração para Desenvolvimento\/Teste/### Development\/Test Configuration/g' \
        -e 's/## 🔧 Parâmetros/## 🔧 Parameters/g' \
        -e 's/## 🔧 Parâmetros de Configuração/## 🔧 Configuration Parameters/g' \
        -e 's/### Entrada (input)/### Input/g' \
        -e 's/### Saída (output)/### Output/g' \
        -e 's/### Entrada (Input)/### Input/g' \
        -e 's/### Saída (Output)/### Output/g' \
        -e 's/## 💻 Exemplos de Uso/## 💻 Usage Examples/g' \
        -e 's/## 🔍 Casos de Uso/## 🔍 Use Cases/g' \
        -e 's/## 🌐 Exemplo Completo/## 🌐 Complete Example/g' \
        -e 's/## 📊 Observabilidade/## 📊 Observability/g' \
        -e 's/## 🔒 Segurança/## 🔒 Security/g' \
        -e 's/## 📈 Performance/## 📈 Performance/g' \
        -e 's/## 🛠️ Implementação/## 🛠️ Implementation/g' \
        -e 's/## 🚨 Tratamento de Erros/## 🚨 Error Handling/g' \
        -e 's/## 🔗 Integração com Outros Módulos/## 🔗 Integration with Other Modules/g' \
        -e 's/## 🧪 Testes/## 🧪 Testing/g' \
        -e 's/## 🔍 Saída de Dados/## 🔍 Data Output/g' \
        -e 's/## 📊 Tipos de Dados Suportados/## 📊 Supported Data Types/g' \
        -e 's/## ⚡ Performance e Boas Práticas/## ⚡ Performance and Best Practices/g' \
        -e 's/## 📈 Monitoramento e Estatísticas/## 📈 Monitoring and Statistics/g' \
        -e 's/## 🔍 Estratégias de TTL/## 🔍 TTL Strategies/g' \
        -e 's/## 💡 Casos de Uso/## 💡 Use Cases/g' \
        -e 's/## 📊 Estrutura de Dados/## 📊 Data Structure/g' \
        -e 's/## 🎨 Saída de Help/## 🎨 Help Output/g' \
        -e 's/## 🛠️ Tratamento de Erros/## 🛠️ Error Handling/g' \
        -e 's/## 🔒 Validação/## 🔒 Validation/g' \
        -e 's/## 🔧 Configuração Avançada/## 🔧 Advanced Configuration/g' \
        -e 's/## 📊 Casos de Uso Comuns/## 📊 Common Use Cases/g' \
        -e 's/### Argumentos Obrigatórios Ausentes/### Missing Required Arguments/g' \
        -e 's/### Flags Desconhecidas/### Unknown Flags/g' \
        -e 's/### Valores Inválidos/### Invalid Values/g' \
        -e 's/### Argumentos Posicionais Inválidos/### Invalid Positional Arguments/g' \
        -e 's/### Uso do Exemplo/### Example Usage/g' \
        -e 's/### Validação de Performance/### Performance Validation/g' \
        -e 's/### Executar Testes do Módulo/### Running Module Tests/g' \
        -e 's/### Métricas Disponíveis/### Available Metrics/g' \
        -e 's/### Interpretação das Métricas/### Metrics Interpretation/g' \
        -e 's/### Complexidade das Operações/### Operation Complexity/g' \
        -e 's/### Padrões de Nomenclatura de Chaves/### Key Naming Patterns/g' \
        -e 's/### Configurações Recomendadas/### Recommended Configurations/g' \
        -e 's/### Erro de Chave Vazia/### Empty Key Error/g' \
        -e 's/### Erro de Ação Inválida/### Invalid Action Error/g' \
        -e 's/### Cache Miss (Não é erro)/### Cache Miss (Not an error)/g' \
        -e 's/### Com HTTP Server/### With HTTP Server/g' \
        -e 's/### Com Database (PostgreSQL)/### With Database (PostgreSQL)/g' \
        -e 's/#### Desenvolvimento/#### Development/g' \
        -e 's/#### Staging/#### Staging/g' \
        -e 's/#### Produção/#### Production/g' \
        -e 's/**Versão**/**Version**/g' \
        -e 's/**Autor**/**Author**/g' \
        -e 's/**Licença**/**License**/g' \
        -e 's/**Repositório**/**Repository**/g' \
        -e 's/obrigatório/required/g' \
        -e 's/opcional/optional/g' \
        -e 's/padrão:/default:/g' \
        -e 's/Padrão:/Default:/g' \
        -e 's/string, obrigatório/string, required/g' \
        -e 's/string, opcional/string, optional/g' \
        -e 's/integer, obrigatório/integer, required/g' \
        -e 's/integer, opcional/integer, optional/g' \
        -e 's/boolean, obrigatório/boolean, required/g' \
        -e 's/boolean, opcional/boolean, optional/g' \
        -e 's/array, obrigatório/array, required/g' \
        -e 's/array, opcional/array, optional/g' \
        -e 's/object, obrigatório/object, required/g' \
        -e 's/object, opcional/object, optional/g' \
        -e 's/any, obrigatório/any, required/g' \
        -e 's/any, opcional/any, optional/g' \
        -e 's/Tipo:/Type:/g' \
        -e 's/Descrição:/Description:/g' \
        -e 's/Obrigatório:/Required:/g' \
        -e 's/# Módulo /# /g' \
        -e 's/O módulo /The /g' \
        -e 's/ módulo / module /g' \
        -e 's/fornece /provides /g' \
        -e 's/funcionalidades /functionality /g' \
        -e 's/## 🚀 Funcionalidades/## 🚀 Features/g' \
        -e 's/### Características Principais/### Key Features/g' \
        -e 's/**Alta Performance**/**High Performance**/g' \
        -e 's/**TTL Automático**/**Automatic TTL**/g' \
        -e 's/**Thread Safety**/**Thread Safety**/g' \
        -e 's/**Observabilidade**/**Observability**/g' \
        -e 's/**Simplicidade**/**Simplicity**/g' \
        -e 's/**Performance**/**Performance**/g' \
        -e 's/**Debug**/**Debug**/g' \
        "$file"
    
    echo "Translation completed for $file"
}

# Translate specific files that have Portuguese content
for file in "$DOCS_DIR"/{http_request,http_server,jwt,log,postgres,rpc,sleep}.md; do
    if [ -f "$file" ]; then
        translate_file "$file"
    fi
done

# Translate remaining Portuguese sections in echo.md
echo "Translating remaining sections in echo.md..."
sed -i \
    -e 's/## 💻 Exemplos de Uso/## 💻 Usage Examples/g' \
    -e 's/### Echo de String Simples/### Simple String Echo/g' \
    -e 's/### Echo de Número/### Number Echo/g' \
    -e 's/### Echo de Boolean/### Boolean Echo/g' \
    -e 's/### Echo de Array/### Array Echo/g' \
    -e 's/### Echo de Objeto Complexo/### Complex Object Echo/g' \
    -e 's/### Echo com Dados Dinâmicos/### Echo with Dynamic Data/g' \
    -e 's/## 🔍 Casos de Uso/## 🔍 Use Cases/g' \
    -e 's/### 1\. Debug de Pipeline/### 1. Pipeline Debug/g' \
    -e 's/### 2\. Passagem de Dados/### 2. Data Passing/g' \
    -e 's/### 3\. Validação de Estruturas/### 3. Structure Validation/g' \
    -e 's/### 4\. Testes e Desenvolvimento/### 4. Testing and Development/g' \
    -e 's/## 🌐 Exemplo Completo/## 🌐 Complete Example/g' \
    -e 's/description: "Demonstração do módulo Echo"/description: "Echo module demonstration"/g' \
    -e 's/# Esta mensagem será ecoada/# This message will be echoed/g' \
    -e 's/# Saída: "Esta mensagem será ecoada"/# Output: "This message will be echoed"/g' \
    -e 's/# Saída: 42/# Output: 42/g' \
    -e 's/# Saída: true/# Output: true/g' \
    -e 's/input: \[1, 2, 3, "teste", true\]/input: [1, 2, 3, "test", true]/g' \
    -e 's/# Saída: \[1, 2, 3, "teste", true\]/# Output: [1, 2, 3, "test", true]/g' \
    -e 's/# Saída: (objeto idêntico ao input)/# Output: (identical object to input)/g' \
    -e 's/# Algum processamento que retorna dados do usuário/# Some processing that returns user data/g' \
    -e 's/# Saída: (dados do usuário do step anterior)/# Output: (user data from previous step)/g' \
    -e 's/# Útil para ver exatamente o que a API retornou/# Useful to see exactly what the API returned/g' \
    -e 's/# Continua processamento\.\.\./# Continue processing.../g' \
    -e 's/input: "Resultado: {{ \$pass_result }}"/input: "Result: {{ \$pass_result }}"/g' \
    -e 's/name: "Usuário Teste"/name: "Test User"/g' \
    -e 's/# Verifica se o objeto foi criado corretamente/# Checks if object was created correctly/g' \
    -e 's/# Processa como se viesse de uma API real/# Process as if it came from a real API/g' \
    "$DOCS_DIR/echo.md"

echo "All translations completed!"
echo "Backup files (.backup) were created for safety."
