---
sidebar_position: 1
title: Git Repositories
---

# Git Repositories

Phlow provides native support for executing flows directly from Git repositories. This allows you to run workflows from remote repositories without manually cloning them, making it perfect for distributed teams and sharing reusable flows.

## Basic Usage

### SSH Authentication

```bash
# Execute flow from a Git repository via SSH
phlow git@github.com:phlowdotdev/phlow-mirror-request.git
```

### HTTPS Authentication

```bash
# Execute flow from a Git repository via HTTPS
phlow https://github.com/phlowdotdev/phlow-mirror-request.git
```

## Branch Selection

You can specify which branch to use by adding `#branch_name` to the repository URL:

```bash
# Clone and execute from the 'develop' branch
phlow git@github.com:phlowdotdev/phlow-mirror-request.git#develop

# Clone and execute from a feature branch
phlow git@github.com:phlowdotdev/phlow-mirror-request.git#feature/new-functionality
```

## File Selection

By default, Phlow looks for these files in the repository root (in order of priority):
1. `main.phlow`
2. `main.yaml`
3. `main.yml`

### Using PHLOW_MAIN_FILE

To execute a specific file within the repository, use the `PHLOW_MAIN_FILE` environment variable:

```bash
# Execute a specific file from the repository
export PHLOW_MAIN_FILE='examples/webhook-handler.phlow'
phlow git@github.com:phlowdotdev/phlow-mirror-request.git

# Or in a single command
PHLOW_MAIN_FILE='examples/webhook-handler.phlow' phlow git@github.com:phlowdotdev/phlow-mirror-request.git
```

### Advanced File Selection Examples

```bash
# Execute a flow from a subdirectory
PHLOW_MAIN_FILE='flows/api/webhook.phlow' phlow git@github.com:your-org/flows-repo.git

# Execute from a specific branch and file
PHLOW_MAIN_FILE='deployment/prod.phlow' phlow git@github.com:your-org/flows-repo.git#production

# Execute a flow with a different extension
PHLOW_MAIN_FILE='workflows/data-processing.yaml' phlow git@github.com:your-org/flows-repo.git
```

## Authentication

### SSH Key Authentication

By default, Phlow uses the SSH key located at `~/.ssh/id_rsa` for Git authentication.

#### Using Default SSH Key

```bash
# Uses ~/.ssh/id_rsa automatically
phlow git@github.com:phlowdotdev/phlow-mirror-request.git
```

#### Using Custom SSH Key

```bash
# Set custom SSH key path
export PHLOW_REMOTE_ID_RSA_PATH=/path/to/your/custom_key
phlow git@github.com:phlowdotdev/phlow-mirror-request.git

# Or in a single command
PHLOW_REMOTE_ID_RSA_PATH=/path/to/your/custom_key phlow git@github.com:phlowdotdev/phlow-mirror-request.git
```

#### SSH Key Requirements

- The SSH key must be in OpenSSH format
- The corresponding public key should be added to your Git provider (GitHub, GitLab, etc.)
- The private key file should have appropriate permissions (600)

## Practical Examples

### Example 1: Running Mirror Request Flow

The [phlow-mirror-request](https://github.com/phlowdotdev/phlow-mirror-request) repository contains a flow that mirrors HTTP requests. Here's how to run it:

```bash
# Run the main flow
phlow git@github.com:phlowdotdev/phlow-mirror-request.git

# Run from HTTPS
phlow https://github.com/phlowdotdev/phlow-mirror-request.git

# Run from a specific branch
phlow git@github.com:phlowdotdev/phlow-mirror-request.git#main
```

### Example 2: Organization Workflows

```bash
# Run a deployment flow from your organization
PHLOW_MAIN_FILE='deployment/staging.phlow' phlow git@github.com:your-org/devops-flows.git

# Run a monitoring flow
PHLOW_MAIN_FILE='monitoring/health-check.phlow' phlow git@github.com:your-org/ops-flows.git#production
```

### Example 3: Development Workflows

```bash
# Test a development branch
PHLOW_MAIN_FILE='test/integration.phlow' phlow git@github.com:your-org/project.git#feature/new-api

# Run a specific workflow
PHLOW_MAIN_FILE='workflows/data-processing.phlow' phlow git@github.com:your-org/project.git
```

## How It Works

1. **Clone**: Phlow clones the repository to a temporary directory (`phlow_remote`)
2. **Checkout**: If a branch is specified, Phlow checks out that branch
3. **Locate**: Phlow searches for the main file (default behavior) or uses `PHLOW_MAIN_FILE`
4. **Execute**: The flow is executed with the repository as the working directory
5. **Cleanup**: The temporary directory is cleaned up after execution

## Environment Variables Summary

| Variable | Description | Default |
|----------|-------------|---------|
| `PHLOW_MAIN_FILE` | Specify exact file path within the repository | Search for default files |
| `PHLOW_REMOTE_ID_RSA_PATH` | Custom SSH private key path | `~/.ssh/id_rsa` |

## Error Handling

### Common Error Messages

```bash
# File not found
Error: Specified file 'invalid/path.phlow' not found in repository 'git@github.com:user/repo.git'

# SSH key not found
Error: SSH key not found at path: /path/to/key
```

### Troubleshooting

1. **SSH Authentication Issues**:
   - Verify SSH key is added to your Git provider
   - Check SSH key permissions (`chmod 600 ~/.ssh/id_rsa`)

### Best Practices

1. **Use Specific Files**: Use `PHLOW_MAIN_FILE` for better organization and clarity
2. **Version Control**: Pin to specific branches or tags for production use
3. **Security**: Use SSH keys for private repositories
4. **Organization**: Structure your flows in logical directories within repositories
5. **Documentation**: Include README files in your flow repositories

This Git integration enables powerful workflows where teams can share, version, and execute flows from centralized repositories while maintaining security and flexibility.

