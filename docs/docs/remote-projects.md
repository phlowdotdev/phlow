---
sidebar_position: 8
title: Running Remote Projects
---

Phlow supports running remote projects directly from URLs or Git repositories. You can pass a `.git`, `.zip`, or `.tar.gz` source — Phlow will automatically download, extract (if needed), and execute the flow from a `main.yaml`.

```bash
# Git via SSH
phlow git@github.com:lowcarboncode/phlow-mirror-request.git 

# Git via HTTPS
phlow https://github.com/lowcarboncode/phlow-mirror-request.git

# ZIP archive
phlow https://github.com/lowcarboncode/phlow-mirror-request/archive/refs/heads/main.zip

# Tarball (GZIP)
phlow https://github.com/lowcarboncode/phlow-mirror-request/tarball/main
```
###  Git branch selector

```bash
phlow git@github.com:lowcarboncode/phlow-mirror-request.git#develop
```

### Custom SSH Key
By default, Phlow uses the SSH key at ~/.ssh/id_rsa to authenticate Git over SSH.
To override this path, set the environment variable:

```bash
export PHLOW_REMOTE_ID_RSA_PATH=/path/to/your/private_key
```

###  Authorization Header for ZIP/GZIP Downloads
When downloading `.zip` or `.tar.gz` files that require authentication (e.g., from a private server), you can use the environment variable below to send an `Authorization` header in the request:

```bash
export PHLOW_REMOTE_HEADER_AUTHORIZATION="Bearer your_token_here"
```

Phlow will include this header when performing the HTTP request for ZIP or GZIP downloads.


###  Inner directory selector (ZIP/GZIP)
If you are downloading a ZIP or GZIP archive and want to specify which folder inside the archive contains your flow, you can add `#folder_name` at the end:

```bash
phlow https://github.com/lowcarboncode/phlow-mirror-request/archive/refs/heads/main.zip#phlow-mirror-request
```

###  Auto-detection of inner folder
If you don’t specify a folder name and the ZIP/GZIP file contains only one directory, Phlow will automatically treat it as the root and search for a `main.yaml` inside it.

If the archive contains multiple folders or any loose files in the root and no folder is specified, Phlow will return an error.
