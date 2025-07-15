---
sidebar_position: 6
title: Git Remote Execution
---

# Git Remote Execution Examples

This guide demonstrates how to execute flows from Git repositories using Phlow, with practical examples using the [phlow-mirror-request](https://github.com/phlowdotdev/phlow-mirror-request) repository.

## Basic Repository Execution

### Default File Execution

```bash
# Execute the default main.phlow file from the repository
phlow git@github.com:phlowdotdev/phlow-mirror-request.git

# Using HTTPS instead of SSH
phlow https://github.com/phlowdotdev/phlow-mirror-request.git
```

### Branch-Specific Execution

```bash
# Execute from a specific branch
phlow git@github.com:phlowdotdev/phlow-mirror-request.git#main

# Execute from a development branch
phlow git@github.com:phlowdotdev/phlow-mirror-request.git#develop
```

## Advanced File Targeting

### Using PHLOW_MAIN_FILE

The `PHLOW_MAIN_FILE` environment variable allows you to specify exactly which file to execute within the repository:

```bash
# Execute a specific file within the repository
export PHLOW_MAIN_FILE='examples/basic-mirror.phlow'
phlow git@github.com:phlowdotdev/phlow-mirror-request.git

# Or in a single command
PHLOW_MAIN_FILE='examples/basic-mirror.phlow' phlow git@github.com:phlowdotdev/phlow-mirror-request.git
```

### Complex Repository Structure

For repositories with multiple workflows:

```bash
# Execute API-specific workflows
PHLOW_MAIN_FILE='api/webhook-handler.phlow' phlow git@github.com:your-org/api-flows.git

# Execute deployment workflows
PHLOW_MAIN_FILE='deployment/staging.phlow' phlow git@github.com:your-org/devops-flows.git

# Execute testing workflows
PHLOW_MAIN_FILE='tests/integration/api-tests.phlow' phlow git@github.com:your-org/test-flows.git
```

## Authentication Examples

### SSH Key Authentication

```bash
# Using default SSH key (~/.ssh/id_rsa)
phlow git@github.com:phlowdotdev/phlow-mirror-request.git

# Using custom SSH key
export PHLOW_REMOTE_ID_RSA_PATH=/path/to/custom/ssh_key
phlow git@github.com:phlowdotdev/phlow-mirror-request.git

# One-line with custom SSH key
PHLOW_REMOTE_ID_RSA_PATH=/path/to/custom/ssh_key phlow git@github.com:phlowdotdev/phlow-mirror-request.git
```

### Combined Authentication and File Selection

```bash
# Execute specific file with custom SSH key
export PHLOW_REMOTE_ID_RSA_PATH=/path/to/custom/ssh_key
export PHLOW_MAIN_FILE='workflows/mirror-advanced.phlow'
phlow git@github.com:phlowdotdev/phlow-mirror-request.git

# One-line version
PHLOW_REMOTE_ID_RSA_PATH=/path/to/custom/ssh_key PHLOW_MAIN_FILE='workflows/mirror-advanced.phlow' phlow git@github.com:phlowdotdev/phlow-mirror-request.git
```

## Real-World Scenarios

### Scenario 1: Shared Utility Execution

```bash
#!/bin/bash
# Shared utility execution script

# Set environment variables
export PHLOW_REMOTE_ID_RSA_PATH="/path/to/ssh/key"
export PHLOW_MAIN_FILE="utilities/data-backup.phlow"

# Execute utility flow
phlow git@github.com:your-org/utility-flows.git#main
```

### Scenario 2: Development Testing

```bash
#!/bin/bash
# Development testing script

# Test different environments
PHLOW_MAIN_FILE='tests/unit.phlow' phlow git@github.com:your-org/test-flows.git#develop
PHLOW_MAIN_FILE='tests/integration.phlow' phlow git@github.com:your-org/test-flows.git#develop
PHLOW_MAIN_FILE='tests/e2e.phlow' phlow git@github.com:your-org/test-flows.git#develop
```

### Scenario 3: Multi-Environment Deployment

```bash
#!/bin/bash
# Multi-environment deployment

REPO="git@github.com:your-org/deployment-flows.git"

# Deploy to staging
PHLOW_MAIN_FILE='deployment/staging.phlow' phlow $REPO#staging

# Deploy to production (after staging success)
if [ $? -eq 0 ]; then
    PHLOW_MAIN_FILE='deployment/production.phlow' phlow $REPO#production
fi
```

## Repository Structure Examples

### Recommended Structure

```
your-flows-repo/
├── main.phlow                  # Default entry point
├── api/
│   ├── webhook-handler.phlow
│   └── rate-limiter.phlow
├── deployment/
│   ├── staging.phlow
│   ├── production.phlow
│   └── rollback.phlow
├── monitoring/
│   ├── health-check.phlow
│   └── metrics-collector.phlow
└── tests/
    ├── unit.phlow
    ├── integration.phlow
    └── e2e.phlow
```

### Usage with Structure

```bash
# Default execution
phlow git@github.com:your-org/flows-repo.git

# API workflows
PHLOW_MAIN_FILE='api/webhook-handler.phlow' phlow git@github.com:your-org/flows-repo.git

# Deployment workflows
PHLOW_MAIN_FILE='deployment/staging.phlow' phlow git@github.com:your-org/flows-repo.git

# Monitoring workflows
PHLOW_MAIN_FILE='monitoring/health-check.phlow' phlow git@github.com:your-org/flows-repo.git

# Test workflows
PHLOW_MAIN_FILE='tests/integration.phlow' phlow git@github.com:your-org/flows-repo.git
```

## Error Handling

### Common Issues and Solutions

```bash
# Issue: File not found
# Error: Specified file 'invalid/path.phlow' not found in repository
# Solution: Verify the file path exists in the repository

# Issue: SSH authentication failed
# Error: Git clone failed: authentication failed
# Solution: Check SSH key permissions and ensure it's added to your Git provider

# Issue: Branch not found
# Error: Branch 'invalid-branch' not found
# Solution: Verify the branch exists in the remote repository
```

### Debug Mode

```bash
# Enable debug logging to troubleshoot issues
export PHLOW_LOG=DEBUG
export PHLOW_YAML_SHOW=true
PHLOW_MAIN_FILE='workflows/debug.phlow' phlow git@github.com:your-org/flows-repo.git
```

## Best Practices

1. **Use Descriptive File Names**: Choose clear, descriptive names for your flow files
2. **Organize by Function**: Group related flows in directories (api/, deployment/, tests/)
3. **Document Your Flows**: Include README files in your repositories
4. **Use Environment Variables**: Set commonly used variables in your environment
5. **Test Locally First**: Test flows locally before using them in production
6. **Pin to Specific Branches**: Use specific branches or tags for production deployments

This Git remote execution capability makes Phlow a powerful tool for distributed teams and automated workflows, allowing you to centralize and version control your flow definitions while maintaining flexibility in execution.

For more details on Git repositories, see the [Running Remote Projects](../remote-projects/remote-projects.md) documentation.

<citations>
<document>
<document_type>WEB_PAGE</document_type>
<document_id>https://github.com/phlowdotdev/phlow-mirror-request</document_id>
</document>
</citations>
