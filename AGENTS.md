# AGENTS

Este repositorio contem o runtime Phlow escrito em Rust, com modulos em `modules/`
e o runtime principal em `phlow-runtime/`. Use `rg` para buscas.

Comandos comuns (raiz do repo):
- Build do runtime: `cargo build --release -p phlow-runtime`
- Rodar exemplo: `cargo run -p phlow-runtime -- examples/<caminho>.phlow`
- Empacotar modulos: `./scripts/packages.sh`

Estrutura relevante:
- `phlow-runtime/`: runtime principal
- `modules/`: modulos locais
- `phlow-sdk/`: SDKs
- `examples/`: fluxos de exemplo
- `scripts/`: scripts auxiliares

Regra:
- Antes de concluir a tarefa, buildar todos os pacotes: `cargo build --workspace`
