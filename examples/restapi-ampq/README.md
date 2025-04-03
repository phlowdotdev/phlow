# Example Http_rest_api + AMPQ Queue producer and comsumer

Execute:

``` 
docker run -d --name phlow-amqp -p 5672:5672 -p 15672:15672 rabbitmq:3-management
```

Run:
```
phlow examples/http_rest_api-queue/api.yaml
```

And, in new terminal
```
phlow examples/http_rest_api-queue/consumer.yaml
```