# Relatorio de analise: phlow-runtime e phlow-engine

Este documento lista pontos de falha, riscos e oportunidades de melhoria em performance e memoria
observados na analise estatica do codigo.

## Falhas (risco funcional)

1. Critico: unload de biblioteca dinamica pode invalidar pointers do plugin.
   - Arquivo: `phlow-runtime/src/loader/mod.rs`
   - Trecho: `ModuleType::Binary` carrega `Library` e chama `func(setup)`; o `Library` cai fora
     de escopo logo depois.
   - Risco: se o plugin mantiver ponteiros/threads, ocorre use-after-free e crash.
   - Sugestao: manter o `Library` vivo enquanto o modulo estiver registrado (ex.: armazenar
     `Arc<Library>` no runtime/Modules).

2. Alto: panics/aborts em entradas invalidas ou ambiente incompleto.
   - Arquivos:
     - `phlow-engine/src/step_worker.rs` (fatiamento `string[..limit]`)
     - `phlow-engine/src/condition.rs` (panic em operador invalido)
     - `phlow-runtime/src/loader/loader.rs` (`unwrap` em parse de YAML)
     - `phlow-runtime/src/loader/loader.rs` (`HOME` ausente para SSH)
     - `phlow-runtime/src/scripts.rs` (`unwrap` e `process::exit`)
   - Risco: crash em runtime por dados inesperados, strings UTF-8, ou env incompleto.
   - Sugestao: trocar `unwrap/panic` por `Result` com erro claro e fallback.

3. Alto: referencias invalidas e termino precoce em pipelines.
   - Arquivos:
     - `phlow-engine/src/transform.rs` (`process_raw_steps` assume mapa nao vazio)
     - `phlow-engine/src/transform.rs` (`get_next_step` retorna step inexistente)
     - `phlow-engine/src/phlow.rs` (sub-pipeline termina sem retorno ao pai)
   - Risco: fluxo termina cedo ou salta para step inexistente.
   - Sugestao: validar pipeline antes de executar e retornar `Option/Result` nas transicoes.

4. Medio: `Runtime::load_modules` pode bloquear indefinidamente.
   - Arquivo: `phlow-runtime/src/runtime.rs`
   - Detalhe: espera `setup_receive.await` sem timeout.
   - Risco: modulo que nao responde trava o runtime.
   - Sugestao: aplicar timeout e logar fallback.

5. Medio: pasta `phlow_remote` apagada a cada load.
   - Arquivo: `phlow-runtime/src/loader/loader.rs`
   - Risco: concorrencia e perda de dados em downloads simultaneos.
   - Sugestao: usar diretorio temporario unico por operacao (ex.: timestamp/uuid).

6. Baixo: debug server exposto em `0.0.0.0` sem autenticacao.
   - Arquivo: `phlow-runtime/src/debug_server.rs`
   - Risco: clientes externos podem inspecionar/pausar execucao.
   - Sugestao: bind em `127.0.0.1` por padrao e exigir token quando exposto.

## Performance

1. Alta: clones grandes a cada execucao de script.
   - Arquivo: `phlow-engine/src/script.rs`
   - Detalhe: `steps/main/payload/input/setup/tests` sao clonados e convertidos em `Dynamic`.
   - Impacto: custo O(n) por step, amplifica em fluxos longos.
   - Sugestao: reutilizar `Scope`, cache de `Dynamic`, ou limitar exposicao de `steps`.

2. Alta: clones do `Context` e `Value` em execucao de modulos.
   - Arquivo: `phlow-engine/src/step_worker.rs`
   - Impacto: payloads grandes aumentam CPU/memoria por step.
   - Sugestao: mutacao in-place do contexto ou uso de `Cow`/`Arc<Value>`.

3. Media: debug compila/serializa output por step.
   - Arquivo: `phlow-engine/src/pipeline.rs` / `phlow-engine/src/step_worker.rs`
   - Impacto: overhead extra em execucoes longas.
   - Sugestao: condicionar a debug ou calcular sob demanda.

4. Media: execucao de pacote com `spawn_blocking` + `block_in_place` + `block_on`.
   - Arquivo: `phlow-runtime/src/runtime.rs`
   - Impacto: reduz throughput e limita paralelismo.
   - Sugestao: executar `phlow.execute` em tasks async nativas.

5. Media: um `thread::spawn` por modulo.
   - Arquivo: `phlow-runtime/src/runtime.rs`
   - Impacto: escala mal com muitos modulos.
   - Sugestao: threadpool ou carregamento sequencial configuravel.

6. Media: downloads paralelos sem limite e buffer completo em memoria.
   - Arquivo: `phlow-runtime/src/loader/mod.rs`
   - Impacto: pico de memoria/CPU em muitos modulos.
   - Sugestao: limitar concorrencia e stream para disco.

## Memoria

1. Alta: historico de debug cresce sem limite.
   - Arquivos: `phlow-engine/src/debug.rs`, `phlow-runtime/src/debug_server.rs`
   - Impacto: uso de memoria cresce com o tempo.
   - Sugestao: ring buffer com limite e paginacao no comando `ALL`.

2. Alta: `Pipeline::execute` guarda payloads de cada step.
   - Arquivo: `phlow-engine/src/pipeline.rs`
   - Impacto: crescimento linear e mais clones em scripts.
   - Sugestao: flag para desabilitar, truncar ou armazenar apenas IDs.

3. Media: `add_uuids` copia o script inteiro em modo debug.
   - Arquivo: `phlow-engine/src/phlow.rs`
   - Impacto: duplicacao de memoria em flows grandes.
   - Sugestao: gerar UUID sob demanda ou manter mapa paralelo.

4. Media: tarball inteiro em memoria antes de extrair.
   - Arquivo: `phlow-runtime/src/loader/mod.rs`
   - Impacto: alto pico de memoria para pacotes grandes.
   - Sugestao: streaming direto para arquivo/decoder.

## Build

- `cargo build --workspace` (ok)
- Testes nao executados (nao solicitados)
