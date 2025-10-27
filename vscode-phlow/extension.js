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
    // single key decoration (unified color) will be created from configuration
    let unifiedKeyDecoration = null;

    const DEFAULT_BG = [
        'rgba(230,57,70,0.05)',   // #E63946
        'rgba(251,133,0,0.05)',   // #FB8500
        'rgba(255,209,102,0.05)', // #FFD166
        'rgba(6,214,160,0.05)',   // #06D6A0
        'rgba(17,138,178,0.05)',  // #118AB2
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

        // helper: pick a fallback foreground if needed
        function pickFallbackFg() { return '#ffffff'; }

        for (const bg of colors) {
            // apply background to the whole line so items appear as solid blocks
            const dt = vscode.window.createTextEditorDecorationType({ backgroundColor: bg, borderRadius: '3px', isWholeLine: true });
            decorationTypes.push(dt);
            context.subscriptions.push(dt);
        }

        // create (or recreate) unified key decoration from configuration
        if (unifiedKeyDecoration) {
            try { unifiedKeyDecoration.dispose(); } catch (e) { /* ignore */ }
            unifiedKeyDecoration = null;
        }
        // const keyColor = vscode.workspace.getConfiguration('phlow').get('keys.color', '#ffffff');
        // const fgColor = keyColor || pickFallbackFg();
        // unifiedKeyDecoration = vscode.window.createTextEditorDecorationType({ color: fgColor });
        // context.subscriptions.push(unifiedKeyDecoration);

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
    const keyRangesPerColor = decorationTypes.map(() => []);
    const flatDashKeyRanges = [];
    const flatNonDashKeyRanges = [];
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
                        // check if this item starts an explicit brace block (e.g. '- payload: {')
                        const lineAfterDash = lines[j].slice(itemMatch[1].length + 2); // content after '- '
                        const firstBraceIdxInLine = lines[j].indexOf('{', itemMatch[1].length + 2);
                        let foundBrace = firstBraceIdxInLine !== -1;
                        if (foundBrace) {
                            // perform simple brace matching across lines (counts { and })
                            let braceCount = 0;
                            let closed = false;
                            let endCol = null;
                            // scan across lines counting braces, but ignore braces inside strings
                            let inString = null; // current string delimiter ('"', '\'', '`') or null
                            let escaped = false; // previous char was backslash
                            for (let k = j; k < lines.length; k++) {
                                const text = lines[k];
                                // start scanning from where we first found '{' on the first line
                                let startIdx = 0;
                                if (k === j) startIdx = firstBraceIdxInLine;
                                for (let p = startIdx; p < text.length; p++) {
                                    const ch = text[p];
                                    if (escaped) {
                                        // escaped char inside a string, skip
                                        escaped = false;
                                        continue;
                                    }
                                    if (ch === '\\') {
                                        // escape next char
                                        escaped = true;
                                        continue;
                                    }
                                    if (inString) {
                                        if (ch === inString) {
                                            // end of string
                                            inString = null;
                                        }
                                        continue; // ignore braces inside strings
                                    } else {
                                        // not inside a string
                                        if (ch === '"' || ch === '\'' || ch === '`') {
                                            inString = ch;
                                            continue;
                                        }
                                        if (ch === '{') braceCount++;
                                        else if (ch === '}') braceCount--;
                                        if (braceCount === 0 && k >= j) {
                                            endLine = k;
                                            endCol = p + 1; // include the closing brace
                                            closed = true;
                                            break;
                                        }
                                    }
                                }
                                if (closed) break;
                            }
                            // if brace block closed, set range from the opening brace to the line with closing brace
                                if (closed) {
                                    // start from beginning of line so whole-line decoration covers the block
                                    const startPos = new vscode.Position(j, 0);
                                    const finalEndCol = (endCol !== null && typeof endCol !== 'undefined') ? endCol : (lines[endLine].indexOf('}') + 1 || lines[endLine].length);
                                    const endPos = new vscode.Position(endLine, finalEndCol);
                                    const range = new vscode.Range(startPos, endPos);
                                    rangesPerColor[colorIdx % rangesPerColor.length].push(range);
                                    colorIdx++;
                                    j = endLine; // skip to the end of the brace block
                                    continue;
                                }
                            // if not closed, fallthrough to indentation-based expansion below
                        }

                        // fallback: include indented child lines as before
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
                        // start at column 0 so whole-line decoration paints the full visible width
                        const startPos = new vscode.Position(j, 0);
                        const endPos = new vscode.Position(endLine, lines[endLine].length);
                        const range = new vscode.Range(startPos, endPos);
                        const idx = colorIdx % rangesPerColor.length;
                        rangesPerColor[idx].push(range);
                        // find key tokens inside the block (e.g. 'name:' 'image:' 'commands:') and add key ranges
                        for (let ln = j; ln <= endLine; ln++) {
                            const textLine = lines[ln];
                            // first, try to match list-item keys like '- id:'
                            const dashKeyMatch = textLine.match(/^(\s*)-\s+([A-Za-z0-9_\-\.]+)\s*:/);
                            if (dashKeyMatch) {
                                const keyIndent = dashKeyMatch[1].length;
                                const keyName = dashKeyMatch[2];
                                const keyStart = keyIndent + 2; // after '- '
                                const keyEnd = keyStart + keyName.length;
                                const keyRange = new vscode.Range(new vscode.Position(ln, keyStart), new vscode.Position(ln, keyEnd));
                                keyRangesPerColor[idx].push(keyRange);
                                flatDashKeyRanges.push(keyRange);
                                continue;
                            }
                            // match keys at start of property lines (optionally preceded by whitespace)
                            const keyMatch = textLine.match(/^(\s*)([A-Za-z0-9_\-\.]+)\s*:/);
                            if (keyMatch) {
                                const keyIndent = keyMatch[1].length;
                                const keyName = keyMatch[2];
                                const keyStart = keyIndent;
                                const keyEnd = keyIndent + keyName.length;
                                const keyRange = new vscode.Range(new vscode.Position(ln, keyStart), new vscode.Position(ln, keyEnd));
                                keyRangesPerColor[idx].push(keyRange);
                                flatNonDashKeyRanges.push(keyRange);
                            }
                        }
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
        // also return flat lists of keys: dash-prefixed and non-dash (so we can let the grammar/style handle dash keys)
        const flatKeyRanges = [].concat(...keyRangesPerColor);
        return { bg: rangesPerColor, keys: keyRangesPerColor, flatKeys: flatKeyRanges, flatDashKeys: flatDashKeyRanges, flatNonDashKeys: flatNonDashKeyRanges };
    }

    function updateDecorationsForEditor(editor) {
        if (!editor) return;
        const doc = editor.document;
        if (!doc || !doc.fileName.endsWith('.phlow')) return;
        const ranges = findStepsItemRanges(doc);
        for (let k = 0; k < decorationTypes.length; k++) {
            editor.setDecorations(decorationTypes[k], ranges.bg[k] || []);
        }
        // apply unified key decoration only to non-dash keys so list-item keys ("- key:") keep their TextMate scope and theme styling
        if (unifiedKeyDecoration) {
            editor.setDecorations(unifiedKeyDecoration, ranges.flatNonDashKeys || []);
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

