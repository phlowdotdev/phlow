---
sidebar_position: 8
title: Running Remote Projects
---

# Running Remote Projects

Phlow supports running remote projects directly from URLs or Git repositories. You can pass a `.git`, `.zip`, or `.tar.gz` source — Phlow will automatically download, extract (if needed), and execute the flow from a `main.phlow`.

:::tip
For detailed Git repository usage including authentication, branch selection, and file targeting, see the [Git Repositories](./git-repositories.md) section.
:::

## Quick Examples

```bash
# Git via SSH
phlow git@github.com:phlowdotdev/phlow-mirror-request.git 

# Git via HTTPS
phlow https://github.com/phlowdotdev/phlow-mirror-request.git

# ZIP archive
phlow https://github.com/phlowdotdev/phlow-mirror-request/archive/refs/heads/main.zip

# Tarball (GZIP)
phlow https://github.com/phlowdotdev/phlow-mirror-request/tarball/main
```

## Supported Remote Sources

### Git Repositories

Git repositories are the most flexible and powerful way to run remote projects. They support:

- **SSH and HTTPS authentication**
- **Branch selection** with `#branch_name`
- **Custom file targeting** with `PHLOW_MAIN_FILE`
- **Private repositories** with proper authentication

```bash
# Basic examples
phlow git@github.com:phlowdotdev/phlow-mirror-request.git
phlow https://github.com/phlowdotdev/phlow-mirror-request.git#main

# With custom file
PHLOW_MAIN_FILE='flows/webhook.phlow' phlow git@github.com:your-org/flows.git
```

### ZIP Archives

ZIP archives are downloaded and extracted automatically:

```bash
# Public ZIP archive
phlow https://github.com/phlowdotdev/phlow-mirror-request/archive/refs/heads/main.zip

# With folder selection
phlow https://github.com/phlowdotdev/phlow-mirror-request/archive/refs/heads/main.zip#phlow-mirror-request
```

### TAR.GZ Archives

Tarball archives are also supported:

```bash
# Public tarball
phlow https://github.com/phlowdotdev/phlow-mirror-request/tarball/main

# With folder selection
phlow https://github.com/phlowdotdev/phlow-mirror-request/tarball/main#phlow-mirror-request
```

## Authentication

### Git Authentication

For Git repositories, authentication is handled automatically:

- **SSH**: Uses your SSH keys (`~/.ssh/id_rsa` by default)
- **HTTPS**: Uses Git's credential system

```bash
# Custom SSH key
export PHLOW_REMOTE_ID_RSA_PATH=/path/to/your/custom_key
phlow git@github.com:private-org/private-repo.git
```

### ZIP/GZIP Authentication

For private archives requiring authentication:

```bash
export PHLOW_REMOTE_HEADER_AUTHORIZATION="Bearer your_token_here"
phlow https://private-server.com/archive.zip
```

## File Selection

### Default Files

By default, Phlow searches for these files in order:
1. `main.phlow`
2. `main.yaml`
3. `main.yml`

### Custom Files

Use `PHLOW_MAIN_FILE` to specify a custom file:

```bash
# For Git repositories
PHLOW_MAIN_FILE='deployment/production.phlow' phlow git@github.com:your-org/flows.git

# For ZIP archives (relative to extracted root)
PHLOW_MAIN_FILE='workflows/api.phlow' phlow https://example.com/flows.zip
```

## Inner Directory Selection (ZIP/GZIP)

For ZIP and GZIP archives, you can specify which folder contains your flow:

```bash
# Specify folder explicitly
phlow https://github.com/phlowdotdev/phlow-mirror-request/archive/refs/heads/main.zip#phlow-mirror-request

# Auto-detection (if only one directory exists)
phlow https://github.com/phlowdotdev/phlow-mirror-request/archive/refs/heads/main.zip
```

### Auto-detection Behavior

- **Single directory**: Automatically used as root
- **Multiple directories or loose files**: Requires explicit folder specification
- **No folder specified with multiple items**: Returns error

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `PHLOW_MAIN_FILE` | Specify exact file path within remote source | Search for default files |
| `PHLOW_REMOTE_ID_RSA_PATH` | Custom SSH private key path | `~/.ssh/id_rsa` |
| `PHLOW_REMOTE_HEADER_AUTHORIZATION` | Authorization header for ZIP/GZIP downloads | _None_ |

## How It Works

1. **Download/Clone**: Phlow downloads or clones the remote source
2. **Extract**: Archives are extracted to a temporary directory
3. **Navigate**: Moves to the correct directory (specified or auto-detected)
4. **Locate**: Finds the main file (default search or `PHLOW_MAIN_FILE`)
5. **Execute**: Runs the flow from the temporary directory
6. **Cleanup**: Removes temporary files after execution

## Use Cases

### Shared Utility Flows

```bash
# Run a shared utility flow
PHLOW_MAIN_FILE='utils/backup-database.phlow' phlow git@github.com:your-org/shared-flows.git#main
```

### Shared Workflows

```bash
# Run a shared utility flow
phlow git@github.com:your-org/shared-flows.git#utilities
```

### Testing and Development

```bash
# Test a specific branch
PHLOW_MAIN_FILE='tests/integration.phlow' phlow git@github.com:your-org/project.git#feature/new-api
```

### One-off Executions

```bash
# Run a flow from a ZIP archive
phlow https://releases.example.com/flows-v1.2.3.zip
```

## Best Practices

1. **Use Git for Version Control**: Git repositories provide the best experience with branching and version control
2. **Specify Custom Files**: Use `PHLOW_MAIN_FILE` for better organization and clarity
3. **Pin to Specific Versions**: Use specific branches or tags for production deployments
4. **Secure Authentication**: Use SSH keys for private repositories
5. **Organize Flows**: Structure your flows logically within repositories

## Common Patterns

### Multi-Environment Deployments

```bash
# Development
PHLOW_MAIN_FILE='deployment/dev.phlow' phlow git@github.com:your-org/deploy.git#develop

# Staging
PHLOW_MAIN_FILE='deployment/staging.phlow' phlow git@github.com:your-org/deploy.git#staging

# Production
PHLOW_MAIN_FILE='deployment/prod.phlow' phlow git@github.com:your-org/deploy.git#main
```

### Utility Flows

```bash
# Database backup
PHLOW_MAIN_FILE='utils/backup.phlow' phlow git@github.com:your-org/db-utils.git

# Log analysis
PHLOW_MAIN_FILE='monitoring/analyze-logs.phlow' phlow git@github.com:your-org/ops-flows.git
```

This remote project capability makes Phlow incredibly flexible for distributed teams, automated workflows, and sharing reusable flow definitions across projects and organizations.

<citations>
<document>
<document_type>WEB_PAGE</document_type>
<document_id>https://github.com/phlowdotdev/phlow-mirror-request</document_id>
</document>
</citations>
