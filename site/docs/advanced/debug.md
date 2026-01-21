---
sidebar_position: 2
title: Debug
---

# Debug do Phlow

O modo debug pausa a execucao antes de cada step e permite inspecionar o contexto (main/payload), o step atual e o historico de execucao via um servidor TCP e o TUI inspector.

## Ativando o debug

O debug eh habilitado por variavel de ambiente. Use `PHLOW_DEBUG=true`:

```bash
PHLOW_DEBUG=true cargo run -p phlow-runtime -- ./examples/qualquer.phlow
```

Por padrao o servidor debug escuta em `0.0.0.0:31400`. Para mudar a porta, use `PHLOW_DEBUG_PORT`:

```bash
PHLOW_DEBUG=true PHLOW_DEBUG_PORT=31400 cargo run -p phlow-runtime -- ./examples/qualquer.phlow
```

## Inspecao com o phlow-tui-inspect

Em outro terminal, conecte o inspector na mesma porta:

```bash
PHLOW_DEBUG_PORT=31400 cargo run -p phlow-tui-inspect
```

O inspector se conecta em `127.0.0.1`, entao para depurar remotamente use tunel/port-forward.

## Comandos principais

Voce pode digitar os comandos diretamente na barra do inspector:

- `STEP` - mostra o step aguardando execucao
- `SHOW` - mostra o script compilado
- `NEXT` - libera um step
- `RELEASE` - libera o pipeline atual
- `ALL` - mostra historico de steps
- `PAUSE` - pausa qualquer liberacao em andamento

Atalhos do inspector (equivalentes aos comandos acima):

- `/n` (Ctrl+n) - NEXT + STEP
- `/a` (Ctrl+a) - NEXT + ALL
- `/r` (Ctrl+r) - RELEASE + ALL
- `/w` (Ctrl+w) - SHOW
- `/g` (Ctrl+g) - STEP

Use `/m` para abrir o resumo de comandos e `ESC` para fechar.

## Observacoes e seguranca

- Quando o debug esta ativo, a execucao pausa a cada step ate receber `NEXT` ou `RELEASE`.
- O servidor debug eh uma porta TCP simples. Use apenas em ambiente confiavel e evite expor para a internet.
