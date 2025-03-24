# Phlow



## Run a supported collector like jaeger in the background
```
docker run -d -p4318:4318 -p4317:4317 -p16686:16686 jaegertracing/all-in-one:latest
```

## Add envs
```
OTEL_RESOURCE_ATTRIBUTES="service.name=YOUR_SERVICE_NAME,service.version=YOUR_SERVICE_VERSION_" OTEL_SERVICE_NAME="YOUR_SERVICE_NAME" 
```

open: [http://localhost:16686](http://localhost:16686)