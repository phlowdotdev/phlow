---
sidebar_position: 1
title: HTTP - Mirror Request
---
This is a simple example of a Phlow that mirrors a request to Phlow. It uses the `http_server` module to handle incoming HTTP requests and returns the request body as a JSON response.

### main.phlow

```yaml
name: Phlow Mirror Request
description: Mirror request to Phlow.
version: 1.0
main: http_server
modules:
  - module: http_server
    # version: latest (optional - defaults to latest)
steps:
  - return:
      status_code: 200
      body: !phs main
      headers:
        Content-Type: application/json
```

Now, you can run the Phlow using the command line. By default, Phlow will look for a `main.phlow` in the current directory:

```bash
phlow main.phlow
```

### Test
You can test the Phlow by sending a request to the server. You can use `curl` or any HTTP client to send a request to the server.


```bash
curl -X POST http://localhost:3000/ \
  -H "Content-Type: application/json" \
  -H "X-Custom-Header: MyHeaderValue" \
  -d '{"key": "value"}'
```

This command sends a POST request to the server running on `localhost:3000` with a JSON body and a custom header. The server will respond with the same JSON body and headers.

### Expected Output
When you send the request, the server will mirror the request details in the response. For example, if you send the JSON body `{"key": "value"}` with custom headers, the response will look similar to this:

```json
{
  "path": "/",
  "uri": "/",
  "client_ip": "127.0.0.1:45606",
  "query_string": "",
  "query_params": {},
  "body": {
    "key": "value"
  },
  "body_size": 15,
  "headers": {
    "content-type": "application/json",
    "x-custom-header": "MyHeaderValue",
    "host": "localhost:3000",
    "connection": "keep-alive",
    "user-agent": "curl/7.68.0",
    "accept": "*/*",
    "content-length": "15"
  },
  "method": "POST"
}
```
