# Spread Syntax no PHS

O PHS agora suporta spread syntax similar ao JavaScript para objetos e arrays. Esta funcionalidade permite combinar e expandir estruturas de dados de forma elegante.

## Spread de Objetos

### Sintaxe Básica
```javascript
let obj1 = #{a: 1, b: 2};
let obj2 = #{c: 3, d: 4};

// Combinando objetos
let combined = #{...obj1, ...obj2};
// Resultado: #{a: 1, b: 2, c: 3, d: 4}
```

### Sobrescrevendo Propriedades
```javascript
let base = #{name: "João", age: 30, active: false};
let updates = #{age: 31, active: true};

let updated = #{...base, ...updates};
// Resultado: #{name: "João", age: 31, active: true}
```

### Misturando com Propriedades Diretas
```javascript
let user = #{id: 1, name: "Maria"};

let userWithRole = #{...user, role: "admin", created_at: "2025-01-01"};
// Resultado: #{id: 1, name: "Maria", role: "admin", created_at: "2025-01-01"}
```

## Spread de Arrays

### Sintaxe Básica
```javascript
let arr1 = [1, 2, 3];
let arr2 = [4, 5, 6];

// Combinando arrays
let combined = [...arr1, ...arr2];
// Resultado: [1, 2, 3, 4, 5, 6]
```

### Inserindo Elementos
```javascript
let numbers = [2, 3, 4];

let extended = [1, ...numbers, 5, 6];
// Resultado: [1, 2, 3, 4, 5, 6]
```

### Casos de Uso Práticos

#### 1. Merging de Configurações
```javascript
let defaultConfig = #{
    timeout: 5000,
    retries: 3,
    debug: false
};

let userConfig = #{
    timeout: 10000,
    debug: true
};

let finalConfig = #{...defaultConfig, ...userConfig};
// Resultado: #{timeout: 10000, retries: 3, debug: true}
```

#### 2. Combinando Permissões
```javascript
let basePermissions = ["read", "write"];
let adminPermissions = ["delete", "admin"];

let allPermissions = [...basePermissions, "update", ...adminPermissions];
// Resultado: ["read", "write", "update", "delete", "admin"]
```

#### 3. Construindo Respostas de API
```javascript
let userData = #{id: 123, name: "Ana", email: "ana@email.com"};
let metadata = #{created_at: "2025-01-01", last_login: "2025-01-10"};

let apiResponse = #{
    success: true,
    data: #{...userData, ...metadata},
    timestamp: "2025-01-10T10:00:00Z"
};
```

## Implementação Técnica

O spread syntax é implementado através de um pré-processador que transforma a sintaxe em chamadas de função:

- `#{...a, b: 2, ...c}` → `__spread_object([a, #{b: 2}, c])`
- `[...a, 1, ...b]` → `__spread_array([a, [1], b])`

As funções `__spread_object` e `__spread_array` são registradas no engine Rhai e fazem a combinação efetiva dos dados.

## Limitações Atuais

1. **Aninhamento Complexo**: Objetos e arrays muito aninhados com spread podem não ser processados corretamente
2. **Performance**: Para estruturas muito grandes, a performance pode ser impactada
3. **Debugging**: Mensagens de erro podem referenciar as funções internas em vez da sintaxe original

## Exemplos de Teste

Veja os testes em `phs/src/script.rs` na função `test_complete_spread_example` para um exemplo completo de uso.
