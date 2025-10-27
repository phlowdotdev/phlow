const vscode = require('vscode');

/**
 * @param {vscode.ExtensionContext} context
 */
function activate(context) {
    console.log('Phlow VSCode extension activated!');

    function handleDocument(doc) {
        if (!doc) return;
        const fileName = doc.fileName || '';
        if (fileName.endsWith('.phlow')) {
            vscode.window.showInformationMessage('Olá — Phlow: hello world! (arquivo .phlow aberto)');
        }
    }

    // Handle already-open documents when the extension activates
    vscode.workspace.textDocuments.forEach(handleDocument);

    // Show message when a new document is opened
    const openListener = vscode.workspace.onDidOpenTextDocument(handleDocument);
    context.subscriptions.push(openListener);
}

function deactivate() {
    // nothing to cleanup for now
}

module.exports = {
    activate,
    deactivate,
};
