---
sidebar_position: 3
title: Install
---
You can easily install Phlow using our ready-to-use shell scripts.

## Requirements:

- 64-bit Linux system ([x86_64-unknown-linux-gnu](https://doc.rust-lang.org/rustc/platform-support.html) target)
- **glibc** version **2.31** or higher (included in **Ubuntu 20.04** and later)


### Install via `curl`

```bash
curl -fsSL https://raw.githubusercontent.com/phlowdotdev/phlow/main/scripts/install-phlow.sh | { bash || true; }
```

### Install via `wget`

```bash
wget -qO- https://raw.githubusercontent.com/phlowdotdev/phlow/main/scripts/install-phlow.sh | { bash || true; }
```
---

## Running with Docker

There are two ways to execute Phlow using Docker:

1. **Pass a file, gzip, zip, or Git repository URL via the `MAIN_FILE` environment variable**:  
    Phlow will download the file and execute it.

    Example:
    ```bash
    docker run -it --rm -e MAIN_FILE=https://example.com/file.zip ghcr.io/phlowdotdev/phlow:latest
    ```

2. **Create a volume and pass the file path via the `MAIN_FILE` environment variable**:  
    Phlow will execute the specified file from the mounted volume.

    Example:
    ```bash
    docker run -it --rm -v "$(pwd)/examples/restapi-ping:/data" -e PHLOW_MAIN=/data/main.yaml -p 3000:3000 phlow
    ```

> ## Extra example:
> **Run a Phlow mirror request**:
>
>   This example demonstrates how to run a Phlow mirror request using Docker. The `MAIN_FILE` environment variable is set to the URL of the main file in the GitHub repository.
>
>    Example:
>    ```bash
>    docker run -it --rm -e MAIN_FILE=https://github.com/phlowdotdev/phlow-mirror-request/archive/refs/heads/main.zip ghcr.io/phlowdotdev/phlow:latest
>    ```

