# Rust Cross Build GitHub Action

Este Action permite compilar projetos Rust multi-plataforma usando `cross-rs`.

## Inputs

- `target` - (obrigatório) Triple do target, ex: `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-musl`
- `project-path` - (opcional) Caminho do projeto Rust. Padrão: `.`

## Exemplo de uso:

```yaml
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: philippeassis/my-cross-build-action@v1
        with:
          target: x86_64-unknown-linux-gnu
          project-path: .
```
