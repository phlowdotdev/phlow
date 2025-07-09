# Postgres JSON Parameters Example

Este exemplo demonstra como usar o módulo postgres com parâmetros no formato JSON em vez de array.

## Principais Mudanças

O módulo postgres foi atualizado para aceitar parâmetros como um objeto JSON, permitindo uma sintaxe mais clara e legível:

### Formato Anterior (Array)
```yaml
- use: postgres
  input:
    query: "INSERT INTO users (id, name, email) VALUES ($1, $2, $3)"
    params: [123, "João", "joao@email.com"]
```

### Formato Atual (JSON Object)
```yaml
- use: postgres
  input:
    query: "INSERT INTO users (id, name, email) VALUES ($1, $2, $3)"
    params:
      id: 123
      name: "João"
      email: "joao@email.com"
```

## Como Executar

1. Configure as variáveis de ambiente do PostgreSQL:
```bash
export POSTGRES_HOST=localhost
export POSTGRES_PORT=5432
export POSTGRES_USER=postgres
export POSTGRES_PASSWORD=postgres
export POSTGRES_DB=test
```

2. Crie a tabela de usuários:
```sql
CREATE TABLE users (
    id INTEGER PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255) NOT NULL
);
```

3. Execute o exemplo:
```bash
phlow run examples/postgres-json-params/main.yaml 1 "João Silva" "joao@email.com"
```

## Vantagens do Formato JSON

- **Legibilidade**: Os parâmetros são claramente identificados por nome
- **Manutenibilidade**: Mais fácil de modificar e debugar
- **Flexibilidade**: Permite diferentes tipos de dados de forma mais clara
- **Documentação**: O código fica autodocumentado

## Tipos de Dados Suportados

O módulo postgres suporta os seguintes tipos de dados nos parâmetros JSON:

- **Integer**: Números inteiros (i32/i64)
- **Float**: Números decimais (f64)
- **Boolean**: Valores verdadeiro/falso
- **String**: Texto
- **Null**: Valores nulos

## Exemplo Completo

O arquivo `main.yaml` contém três exemplos práticos:

1. **INSERT**: Inserir novo usuário
2. **SELECT**: Buscar usuário por ID e nome
3. **UPDATE**: Atualizar email do usuário

Cada exemplo utiliza parâmetros JSON para demonstrar as diferentes operações com o banco de dados.
