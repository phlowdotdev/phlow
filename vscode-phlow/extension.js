const vscode = require('vscode');

/**
 * @param {vscode.ExtensionContext} context
 */
function activate(context) {
    console.log('Phlow VSCode extension activated!');

    // simple hello message when a .phlow file is opened
    function handleDocument(doc) {
        if (!doc) return;
        const fileName = doc.fileName || '';
        if (fileName.endsWith('.phlow')) {
            vscode.window.showInformationMessage('Olá — Phlow: hello world! (arquivo .phlow aberto)');
        }
    }

    // Decoration types are created from configuration so users can customize colors
    let decorationTypes = [];

    const DEFAULT_BG = [
        'rgba(230,57,70,0.15)',   // #E63946
        'rgba(251,133,0,0.15)',   // #FB8500
        'rgba(255,209,102,0.15)', // #FFD166
        'rgba(6,214,160,0.12)',   // #06D6A0
        'rgba(17,138,178,0.12)',  // #118AB2
    ];

    function createDecorationTypesFromConfig() {
        // dispose old ones
        if (decorationTypes && decorationTypes.length) {
            decorationTypes.forEach(d => {
                try { d.dispose(); } catch (e) { /* ignore */ }
            });
        }
        decorationTypes = [];

        const cfg = vscode.workspace.getConfiguration('phlow');
        const colors = cfg.get('stepsRainbow.colors', DEFAULT_BG);
        if (!Array.isArray(colors) || colors.length === 0) {
            colors = DEFAULT_BG.slice();
        }

        for (const bg of colors) {
            const dt = vscode.window.createTextEditorDecorationType({ backgroundColor: bg, borderRadius: '3px' });
            decorationTypes.push(dt);
            // keep for disposal when extension deactivates
            context.subscriptions.push(dt);
        }

        // re-apply decorations to the active editor
        const activeEditor = vscode.window.activeTextEditor;
        if (activeEditor) updateDecorationsForEditor(activeEditor);
    }

    // Create decorationTypes initially
    createDecorationTypesFromConfig();

    // monitor configuration changes to update colors live
    context.subscriptions.push(vscode.workspace.onDidChangeConfiguration(e => {
        if (e.affectsConfiguration('phlow.stepsRainbow.colors') || e.affectsConfiguration('phlow')) {
            createDecorationTypesFromConfig();
        }
    }));

    function findStepsItemRanges(doc) {
        const rangesPerColor = decorationTypes.map(() => []);
        const lines = doc.getText().split(/\r?\n/);
        for (let i = 0; i < lines.length; i++) {
            const line = lines[i];
            const stepsMatch = line.match(/^\s*steps\s*:/);
            if (stepsMatch) {
                const baseIndent = line.match(/^\s*/)[0].length;
                // scan following lines for items belonging to this steps block
                let colorIdx = 0;
                for (let j = i + 1; j < lines.length; j++) {
                    const l = lines[j];
                    // stop if we reach a line with indent less-or-equal than base and not empty
                    const leading = l.match(/^\s*/)[0].length;
                    if (l.trim() === '') {
                        // blank lines inside block are ok
                        continue;
                    }
                    if (leading <= baseIndent && !l.match(/^\s*-\s+/)) {
                        break;
                    }

                    const itemMatch = l.match(/^(\s*)-\s+(.*)$/);
                    if (itemMatch) {
                        const indent = itemMatch[1].length;
                        // include the whole list item and its child/indented lines (recursive properties)
                        let endLine = j;
                        for (let k = j + 1; k < lines.length; k++) {
                            const next = lines[k];
                            const leadingNext = next.match(/^\s*/)[0].length;
                            // include the line if it's indented deeper than the item (child property)
                            if (next.trim() === '') {
                                // include blank line only if it's indented deeper than the item
                                if (leadingNext > indent) {
                                    endLine = k;
                                    continue;
                                }
                                break;
                            }
                            if (leadingNext > indent) {
                                endLine = k;
                                continue;
                            }
                            // stop if we hit another sibling item or a line at same/lower indent
                            break;
                        }
                        const startPos = new vscode.Position(j, itemMatch[1].length); // include the dash
                        const endPos = new vscode.Position(endLine, lines[endLine].length);
                        const range = new vscode.Range(startPos, endPos);
                        rangesPerColor[colorIdx % rangesPerColor.length].push(range);
                        colorIdx++;
                        // advance j to endLine so outer loop continues after the item's block
                        j = endLine;
                    } else {
                        // if it's not a list item, but indented deeper, keep scanning
                        if (leading > baseIndent) {
                            continue;
                        } else {
                            break;
                        }
                    }
                }
            }
        }
        return rangesPerColor;
    }

    function updateDecorationsForEditor(editor) {
        if (!editor) return;
        const doc = editor.document;
        if (!doc || !doc.fileName.endsWith('.phlow')) return;
        const rangesPerColor = findStepsItemRanges(doc);
        for (let k = 0; k < decorationTypes.length; k++) {
            editor.setDecorations(decorationTypes[k], rangesPerColor[k]);
        }
    }

    // Update decorations for the currently active editor
    const active = vscode.window.activeTextEditor;
    if (active) updateDecorationsForEditor(active);

    // Register listeners to keep decorations in sync
    context.subscriptions.push(vscode.window.onDidChangeActiveTextEditor(editor => updateDecorationsForEditor(editor)));
    context.subscriptions.push(vscode.workspace.onDidChangeTextDocument(e => {
        const editor = vscode.window.activeTextEditor;
        if (editor && e.document === editor.document) updateDecorationsForEditor(editor);
    }));

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

