# AWS module (S3 + SQS)

Este módulo expõe ações de alto nível para S3 e SQS.

A partir desta versão, inclui criação e remoção de buckets S3.

## Exemplos de uso

As chamadas abaixo mostram apenas o `input` do step `aws`.

### Criar bucket

```
{
  "action": "s3",
  "method": "create_bucket",
  "bucket": "meu-bucket-exemplo",
  "location": "us-east-1" // opcional. Para regiões != us-east-1, informe a região
}
```

Observações:
- Para `us-east-1`, o S3 exige que não seja enviado `LocationConstraint`, então o campo é ignorado se você informá-lo como `us-east-1`.
- Para outras regiões válidas (ex.: `us-west-2`, `eu-west-1`), o módulo envia o `CreateBucketConfiguration` automaticamente.

### Deletar bucket

```
{
  "action": "s3",
  "method": "delete_bucket",
  "bucket": "meu-bucket-exemplo",
  "force": true // opcional: se true, esvazia o bucket (objetos/versões) antes de remover
}
```

### Listar buckets

```
{
  "action": "s3",
  "method": "list_buckets"
}
```

## Credenciais, região e endpoint

Região agora é obrigatória via `with.region` e será usada como padrão para operações (ex.: `create_bucket`).
Credenciais/endpoint podem ser configurados via `with` do step, variáveis de ambiente ou cadeia padrão do SDK.
Exemplo de `with` útil para LocalStack/MinIO:

```
with: {
  "region": "us-east-1", // obrigatório
  "access_key_id": "test",
  "secret_access_key": "test",
  "endpoint_url": "http://localhost:4566",
  "s3_force_path_style": true
}
```

## Saída padrão

Todas as ações retornam um objeto no formato:

```
{ "success": true, "data": { ... } }
```

ou, em caso de erro:

```
{ "success": false, "error": "mensagem" }
```

### SQS: deletar fila com force

```
{
  "action": "sqs",
  "method": "delete_queue",
  "queue_url": "<url-da-fila>",
  "force": true // opcional: purga a fila antes de deletar
}
```
