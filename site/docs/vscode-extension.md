# VS Code Extension

The official Phlow VS Code extension provides a rich development experience for building Phlow applications with syntax highlighting, IntelliSense, and enhanced productivity features.

## Installation

### From VS Code Marketplace

1. Open VS Code
2. Go to the Extensions view (`Ctrl+Shift+X` or `Cmd+Shift+X`)
3. Search for "Phlow"
4. Click Install on the "Phlow" extension by Phlow

### From Command Line

```bash
code --install-extension phlow.phlow
```

### Links

- **Marketplace**: [phlow.phlow](https://marketplace.visualstudio.com/items?itemName=phlow.phlow)
- **Repository**: [phlowdotdev/vscode](https://github.com/phlowdotdev/vscode)

## Features

### Syntax Highlighting

The extension provides comprehensive syntax highlighting for Phlow files (`.phlow` extension):

- **Keywords**: `main`, `modules`, `steps`, `use`, `input`, `condition`, `then`, `else`, `return`
- **Directives**: `!include`, `!phs`, `!import`, `!arg`
- **Operators**: `==`, `!=`, `>`, `<`, `>=`, `<=`, `&&`, `||`
- **Functions**: Built-in Phlow functions and expressions
- **Comments**: Single-line and multi-line comments
- **Strings**: Quoted strings with proper escaping
- **Numbers**: Integer and floating-point numbers

### IntelliSense

Get intelligent code completion and suggestions:

- **Module names**: Auto-complete available module names
- **Keywords**: Suggestions for Phlow keywords and directives
- **Structure**: Auto-complete for common Phlow structures
- **Validation**: Real-time syntax validation and error highlighting

### Code Snippets

Pre-built code snippets for common Phlow patterns:

- **Basic Flow**: `phlow-basic` - Creates a basic Phlow structure
- **HTTP Server**: `phlow-http` - HTTP server module setup
- **Condition**: `phlow-condition` - Conditional logic block
- **Module**: `phlow-module` - Module definition
- **Steps**: `phlow-steps` - Steps array structure

### File Association

The extension automatically associates `.phlow` files with Phlow syntax highlighting and features.

## Usage

### Creating a New Phlow File

1. Create a new file with `.phlow` extension
2. Start typing - IntelliSense will suggest completions
3. Use snippets by typing the snippet prefix and pressing `Tab`

### Example

```phlow
main: http_server
modules:
  - module: http_server
    version: latest
  - module: echo
    version: latest

steps:
  - use: echo
    input:
      message: "Hello, World!"
  - return:
      status_code: 200
      body:
        message: !phs payload.message
```

## Configuration

The extension can be configured through VS Code settings:

### Settings

- `phlow.validateOnSave`: Enable validation on save (default: `true`)
- `phlow.autoComplete`: Enable auto-completion (default: `true`)
- `phlow.snippets`: Enable code snippets (default: `true`)

### Accessing Settings

1. Open VS Code Settings (`Ctrl+,` or `Cmd+,`)
2. Search for "Phlow"
3. Configure the available options

## Theme Support

The extension works with all VS Code themes and provides consistent syntax highlighting across:

- Light themes
- Dark themes
- High contrast themes
- Custom themes

## Troubleshooting

### Common Issues

#### Extension Not Working

1. Ensure the file has `.phlow` extension
2. Reload VS Code window (`Ctrl+Shift+P` → "Developer: Reload Window")
3. Check if the extension is enabled in the Extensions view

#### Syntax Highlighting Issues

1. Verify the file is recognized as Phlow (check bottom-right corner of VS Code)
2. If not, manually set the language mode to "Phlow"
3. Report issues on the [GitHub repository](https://github.com/phlowdotdev/vscode/issues)

#### IntelliSense Not Working

1. Check if `phlow.autoComplete` is enabled in settings
2. Restart VS Code
3. Ensure you're using the latest version of the extension

## Contributing

The extension is open source and welcomes contributions:

1. **Repository**: [phlowdotdev/vscode](https://github.com/phlowdotdev/vscode)
2. **Issues**: Report bugs or request features on GitHub
3. **Pull Requests**: Submit improvements and fixes

### Development Setup

1. Clone the repository
2. Install dependencies: `npm install`
3. Open in VS Code
4. Press `F5` to launch Extension Development Host
5. Test your changes

## Changelog

See the [CHANGELOG.md](https://github.com/phlowdotdev/vscode/blob/main/CHANGELOG.md) in the repository for version history and updates.

## Support

- **Documentation**: [https://phlow.dev](https://phlow.dev)
- **Issues**: [GitHub Issues](https://github.com/phlowdotdev/vscode/issues)
- **Community**: Join the Phlow community discussions
