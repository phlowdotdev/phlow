/*
Manipula yaml.

!include [file]: Inclui um arquivo yaml.
!eval [expression]: Adiciona {{expression}} ao yaml.

```yaml
main: restapi
modules: !include modules.yaml
steps:
- id: echo
  module: echo
  input:
    message: Hello, World!
- module: log
  input:
    level: warn
    message: XXXXXXXX
- module: log
  input:
    level: error
    message: exit
- return:
    status_code: 201
    body:
      echo: !eval steps.echo
      main: !eval
        if 10 > 5 {
          return 10
        } else {
          return 5
        }
    headers:
      Content-Type: application/json
```
*/
