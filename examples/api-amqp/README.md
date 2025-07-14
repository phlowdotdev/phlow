# Example http_server + AMPQ Queue producer and comsumer

Execute:

``` 
docker run -d --name phlow-amqp -p 5672:5672 -p 15672:15672 rabbitmq:3-management
```

Run:
```
phlow examples/http_server-queue/api.phlow
```

And, in new terminal
```
phlow examples/http_server-queue/consumer.phlow
```