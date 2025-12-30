# Plano detalhado de migracao Valu3 -> Serde (Value)

Este documento descreve um plano detalhado para migrar o projeto do Valu3
para Serde (serde_json) considerando o uso extensivo de `Value` em runtime,
engine, SDK, scripts e modulos.

## Objetivo
- Substituir Valu3 por serde_json como representacao primaria de `Value`.
- Manter compatibilidade funcional das APIs publicas e comportamento do runtime.
- Reduzir o uso de APIs Valu3 a zero ou manter apenas onde inevitavel (ex.: quickleaf).

## Nao objetivos
- Nao redesenhar o modelo de dados ou o DSL.
- Nao alterar a semantica de pipelines/steps.
- Nao reescrever modulos sem necessidade.

## Decisoes iniciais (obrigatorio antes do codigo)
1) Tipo canonico de Value:
   - Recomendado: `serde_json::Value` como base.
2) Semantica de `Undefined`:
   - Valu3 tem `Value::Undefined`. Serde nao tem.
   - Opcoes:
     - A) criar um wrapper `PhlowValue` com variante `Undefined`.
     - B) mapear `Undefined` para `Null` e tratar com metadados/flags.
   - Recomendacao: Opcao A (preserva comportamento, minimiza diff).
3) Quickleaf:
   - `modules/cache` usa `quickleaf::valu3::Value`.
   - Opcoes:
     - A) manter conversao entre PhlowValue e quickleaf::valu3::Value.
     - B) substituir quickleaf ou adicionar adaptador interno.
4) Compatibilidade de string/number:
   - Confirmar como Valu3 trata `to_string()` (valor vs JSON).
   - Definir helper `as_string()` para manter a mesma expectativa.

## Inventario de uso de Value (resumo)
### Core
- `phlow-sdk`: `context.rs`, `id.rs`, `structs/*`, `macros.rs`, `prelude.rs`.
- `phlow-engine`: `transform.rs`, `condition.rs`, `step_worker.rs`, `collector.rs`, `script.rs`.
- `phlow-runtime`: `loader/*`, `runtime.rs`, `analyzer.rs`, `test_runner.rs`, `scripts.rs`.
- `phs`: `script.rs`, `variable.rs`, `functions.rs`, `lib.rs`, `repositories.rs`.

### Modulos (modules/)
- amqp, aws, cache, cli, echo, fs, http_request, http_server, jwt, log,
  openai, postgres, rpc, sleep e outros.

### Docs
- `PHLOW_MODULE_GUIDE.md`, `site/docs/*`, READMEs de modulos.

## Inventario detalhado (resultado de rg)
### Dependencias diretas de valu3
- `Cargo.toml`, `phlow-sdk/Cargo.toml`, `phlow-engine/Cargo.toml`, `phs/Cargo.toml`.

### Reexports e pontos de entrada
- `phlow-sdk/src/prelude.rs` reexporta `valu3::prelude::*` e `json`.
- `phlow-sdk/src/ext.rs` reexporta `valu3`.
- `phlow-sdk/src/macros.rs` usa `valu3::value::Value` nos channels.

### Uso real de APIs Valu3 (principais)
- `Value::Undefined`: `phs/src/functions.rs`, `phlow-runtime/src/runtime.rs`, `phlow-runtime/src/test_runner.rs`, `phlow-runtime/src/scripts.rs`, `modules/http_request/src/lib.rs`, `modules/http_server/src/router.rs`, `modules/http_server/src/resolver.rs`, `modules/http_server/src/openapi.rs`.
- `Value::json_to_value`: `phs/src/functions.rs`, `phlow-runtime/src/loader/mod.rs`, `phlow-runtime/src/loader/loader.rs`, `phlow-runtime/src/runtime.rs`, `modules/http_request/src/request.rs`, `modules/jwt/src/jwt_handler.rs`, `modules/http_server/src/resolver.rs`, `modules/http_server/src/openapi.rs` (inclui testes).
- `JsonMode`/`to_json`: `phs/src/functions.rs`, `phlow-engine/src/step_worker.rs`, `phlow-engine/src/transform.rs`, `phlow-engine/src/collector.rs`, `phlow-runtime/src/analyzer.rs`, `phlow-runtime/src/runtime.rs`, `modules/http_server/src/setup.rs`, `modules/http_server/src/response.rs`, `modules/http_server/src/openapi.rs`.
- `to_json_inline`: `phs/src/script.rs`, `phlow-runtime/src/preprocessor.rs`.
- `as_string`: `phlow-engine/src/step_worker.rs`, `phlow-runtime/src/analyzer.rs`, `phlow-runtime/src/test_runner.rs`, `modules/amqp/src/setup.rs`, `modules/cache/src/input.rs`, `modules/jwt/src/input.rs`, `modules/jwt/src/config.rs`, `modules/http_server/src/setup.rs`.
- `as_bool`/`to_i64`/`to_u64`/`to_f64`/`as_number`: usados em `phs/src/variable.rs` e em modulos `aws`, `amqp`, `cache`, `http_server/openapi`, `sleep`, `rpc`, `http_request/config`, `fs`.
- `Array::new` e `.values`: `phlow-runtime/src/loader/loader.rs` cria `Value::Array(Array::new())`; `as_array().values` em `phlow-runtime/src/test_runner.rs` e `phlow-runtime/src/analyzer.rs`.
- Pattern matching por variantes `Value::Object/Array/...`: intenso em `phs`, `phlow-engine/src/transform.rs`, `modules/http_server/src/openapi.rs` e `modules/http_server/src/resolver.rs`.
- `Value::from`/`to_value()` para primitives/HashMap/Vec: uso massivo em `phs`, `phlow-engine`, `phlow-runtime`, `modules`.
- `ToValueBehavior`/derive `ToValue`/`FromValue`:
  - `ToValueBehavior`: `phlow-sdk/src/id.rs`, `phlow-sdk/src/structs/mod.rs`, `phlow-engine/src/condition.rs`, `phlow-engine/src/transform.rs`, `phlow-engine/src/collector.rs`, `modules/openai/src/setup.rs`.
  - `derive(ToValue)`: `phlow-engine/src/step_worker.rs`, `modules/amqp/src/produce.rs`, `modules/http_server/src/response.rs`, `modules/rpc/src/service.rs`, `modules/postgres/src/response.rs`.
  - `derive(FromValue)`: `phlow-sdk/src/structs/modules.rs`.
- `Object`/`Array` tipos da prelude: `modules/http_server/src/openapi.rs` usa `Object` diretamente.
- `valu3::Error`: `phlow-runtime/src/loader/error.rs`, `phlow-engine/src/transform.rs`, `modules/http_request/src/request.rs`.

## Mapeamento de APIs Valu3 -> Serde (exemplos)
- `Value::json_to_value(s)` -> `serde_json::from_str::<Value>(s)`
- `value.to_json(JsonMode::Indented)` -> `serde_json::to_string_pretty(&value)`
- `value.to_json(JsonMode::Inline)` -> `serde_json::to_string(&value)`
- `value.to_json_inline()` -> helper local (`to_json(JsonMode::Inline)`)
- `Value::Undefined` -> `PhlowValue::Undefined` (wrapper) ou fallback para Null
- `value.as_string()` -> `value.as_str().unwrap_or_default().to_string()`
- `value.as_array().values` -> `value.as_array().iter()`
- `value.as_number()` -> `value.as_number()` (Number) ou `value.as_i64/as_f64`
- `Value::Number` -> `serde_json::Value::Number`
- `NumberBehavior::to_i64` -> `value.as_i64()`
- `ToValueBehavior` -> trait local `ToValue`
- `#[derive(ToValue, FromValue)]` -> `#[derive(Serialize, Deserialize)]` + `serde_json::to_value/from_value`
- `Value::from(T)` -> `serde_json::to_value(T)` ou `impl From<T> for Value`
- `Value::Array(Array::new())` -> `Value::Array(vec![])` ou `Array::new()` no wrapper

## Plano faseado (detalhado)

### Fase 1: Mapeamento completo e classificacao
- Executar `rg -n "valu3|\\bValue\\b"` no repo todo.
- Catalogar APIs Valu3 usadas:
  - `Value::Undefined`, `json_to_value`, `to_json`, `to_value`, `as_string`,
    `NumberBehavior`, `StringBehavior`, derives `ToValue/FromValue`.
- Classificar por impacto:
  - Critico: runtime/engine/SDK.
  - Medio: modulos com IO/HTTP.
  - Baixo: docs e exemplos.
- Identificar pontos de parse/serialize (JSON/YAML) e decidir o comportamento.
- Resultado atual: ver "Inventario detalhado (resultado de rg)".

### Fase 2: Criar camada de compatibilidade (phlow-sdk)
1) Criar `phlow-sdk/src/value.rs` com:
   - Tipo `Value` (alias ou wrapper).
   - Tipos `Object` e `Array` (alias ou wrappers) para usos diretos.
   - Trait `ToValue` e `FromValue`.
   - `JsonMode` (Inline/Indented) e `to_json` equivalente.
   - Helpers: `as_string`, `as_str`, `as_number`, `as_array`, `to_json_inline`,
     `to_i64`, `to_u64`, `to_f64`, `as_bool`, `is_object`, `is_array`,
     `is_string`, `is_null`, etc.
   - `Array` com `.values` (ou ajuste nos usos) para manter compatibilidade.
   - Implementar `From<T>` para primitives/Vec/HashMap/Option usados no repo.
   - Implementar `PartialEq`/`PartialOrd`/`Display` conforme uso atual do Value.
2) Decidir e implementar `Undefined`:
   - Se wrapper:
     - `enum PhlowValue { Undefined, Json(serde_json::Value) }`
     - Implementar conversoes e helpers para reduzir diff.
3) Ajustar reexports:
   - `phlow-sdk/src/prelude.rs` -> exportar `Value`, `ToValue`, `json!`.
   - `phlow-sdk/src/lib.rs` e `ext.rs` -> remover `valu3` e expor `serde_json` se necessario.
4) Atualizar macros:
   - `phlow-sdk/src/macros.rs` usar o novo `Value`.

### Fase 3: Migrar core (SDK, engine, runtime, phs)
#### phlow-sdk
- `context.rs`: substituir imports e metodos conforme `Value` novo.
- `id.rs`: trocar `ToValueBehavior` por `ToValue`.
- `structs/mod.rs` e `structs/modules.rs`:
  - Remover derives Valu3 e usar `serde` ou conversao manual.
  - Ajustar `to_json` e `to_value`.
- `macros.rs`: substituir `valu3::value::Value` nos channels.

#### phlow-engine
- `transform.rs`: substituir `valu3::Error`, `to_json`, `to_value`.
- `condition.rs`: migrar `StringBehavior`, `ToValueBehavior`.
- `step_worker.rs`: ajustar testes e conversoes.
- `collector.rs` e `script.rs`: ajustes de `Value`.

#### phlow-runtime
- `loader/error.rs`: trocar `valu3::Error` por `serde_json::Error` (ou erro proprio).
- `loader/loader.rs`: trocar `json_to_value` e `JsonMode`.
- `runtime.rs`: ajuste de `Value::json_to_value`, `Undefined`.
- `analyzer.rs`: ajustar `to_json`, `as_array`, `as_string`.
- `test_runner.rs`, `scripts.rs`: ajustes de `Undefined`, `to_value`.
- `preprocessor.rs`: ajustar `to_json_inline`.
- `loader/mod.rs`: atualizar `json` macro e `json_to_value` usados em modulo externo.

#### phs
- `script.rs`: ajustes em `Value::Object/Array` e conversoes.
- `variable.rs`: numericos e comparacoes.
- `functions.rs`: parser de JSON, `JsonMode`, `Value` match.
- `lib.rs` e `repositories.rs`: imports e conversoes.

### Fase 4: Migrar modulos (modules/)
1) Padrao:
   - Atualizar `use phlow_sdk::prelude::*` para o novo `Value`.
   - Substituir `Value::json_to_value` por `serde_json::from_str`.
   - Ajustar `Value::Undefined` se existir.
2) Modulos com pontos especiais:
   - `modules/cache`: converter `Value` para `quickleaf::valu3::Value` ou trocar dependencia.
   - `modules/http_server`: usa `Value::Undefined`, `json_to_value`, `to_json`, `as_number`, pattern matching.
   - `modules/http_request`: usa `Value::json_to_value` e mapeia `valu3::Error`.
   - `modules/amqp`: conversao manual `Value` -> `serde_json::Value` e `as_string`.
   - `modules/jwt`: usa `serde_json` + `Value::json_to_value`.
   - `modules/openai`: conversoes `serde_json::Value` <-> `Value` e `ToValueBehavior`.
   - `modules/postgres`: `Value::from` para tipos SQL.
   - `modules/aws`: uso intenso de `as_bool`, `to_i64`, `as_array`.

### Fase 5: Atualizar docs e exemplos
- Remover referencias diretas a Valu3 em:
  - `PHLOW_MODULE_GUIDE.md`
  - `site/docs/*`
  - `phlow-engine/README.md`
  - `modules/jwt/README.md`
- `README.md` e READMEs de modulos
- Ajustar explicacoes de `Value` e `Undefined`.

### Fase 6: Validacao e limpeza
- Build:
  - `cargo build --release -p phlow-runtime`
  - build dos modulos criticos (http_server, http_request, amqp, cache).
- Rodar exemplos:
  - `cargo run -p phlow-runtime -- examples/<caminho>.phlow`
- Testes:
  - `phs` e `phlow-engine` (se houver).
- Remover dependencia `valu3` dos `Cargo.toml` e garantir `Cargo.lock` limpo.

## Checklist de migracao por componente
### phlow-sdk
- [ ] Criar `value.rs` com Value/ToValue/FromValue/JsonMode/Undefined
- [ ] Expor `Object`/`Array`, `to_json_inline`, `as_number`, `as_str`
- [ ] Garantir `PartialOrd`/`Display` e `From<T>` para tipos usados
- [ ] Atualizar `prelude.rs`, `lib.rs`, `ext.rs`
- [ ] Atualizar `macros.rs`
- [ ] Ajustar `context.rs`, `id.rs`, `structs/*`

### phlow-engine
- [ ] Atualizar `transform.rs` (errors, to_json)
- [ ] Atualizar `condition.rs` (ToValue, string helpers)
- [ ] Atualizar `step_worker.rs` e testes
- [ ] Atualizar `collector.rs`, `script.rs`

### phlow-runtime
- [ ] Atualizar `loader/*` (json_to_value, errors)
- [ ] Atualizar `runtime.rs` (var-main parse)
- [ ] Atualizar `analyzer.rs` e `test_runner.rs`
- [ ] Atualizar `scripts.rs`
- [ ] Atualizar `preprocessor.rs` (to_json_inline)

### phs
- [ ] Atualizar `script.rs`, `variable.rs`, `functions.rs`
- [ ] Atualizar `lib.rs` e `repositories.rs`

### modules/
- [ ] Ajustar parse JSON e uso de Value
- [ ] Resolver `quickleaf` (cache)
- [ ] Revalidar modulos com `Value::Undefined`

### Docs
- [ ] Atualizar guia de modulos e docs do site
- [ ] Remover mencoes a Valu3

## Riscos e mitigacoes
- `Undefined` vs `Null`: risco de regressao em fluxos que dependem de `Undefined`.
  - Mitigar com wrapper ou helper dedicado.
- `to_string()` diferente:
  - Valu3 pode retornar valor bruto; serde retorna JSON.
  - Criar helper `as_string` e revisar locais criticos.
- Ordenacao/comparacao:
  - `phs/src/variable.rs` usa `>`/`<` em `Value`; serde_json::Value nao implementa isso.
  - Implementar `PartialOrd` no wrapper ou reescrever comparacoes.
- Numericos:
  - serde_json::Number nao guarda tipo inteiro vs float.
  - Verificar comparacoes em `variable.rs` e `condition.rs`.
- `as_array().values` e `Array::new`:
  - Ajustar chamadores ou criar `Array` wrapper com `.values`.
- Compatibilidade com quickleaf:
  - Garantir conversor e testes basicos de cache.

## Entregaveis esperados
- Camada `phlow-sdk/src/value.rs` com helpers.
- Remocao de `valu3` dos crates (exceto se necessario para quickleaf).
- Modulos ajustados e docs atualizados.
- Build e exemplos funcionando sem regressao.
