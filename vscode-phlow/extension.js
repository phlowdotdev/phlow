const vscode = require('vscode');

/**
 * @param {vscode.ExtensionContext} context
 */
function activate(context) {
    console.log('Phlow VSCode extension activated!');

    // Output channel for analyzer logs
    const analyzerOutput = vscode.window.createOutputChannel('Phlow Analyzer');
    context.subscriptions.push(analyzerOutput);

    // In-memory store mapping main.phlow path -> analyzer result
    const projects = new Map();

    // Debounce timer for filesystem changes
    let changeDebounce = null;
    const CHANGE_DEBOUNCE_MS = 500;

    // Helper: run `phlow <dir> --analyzer --all --json` and parse JSON
    const cp = require('child_process');
    const path = require('path');

    async function runAnalyzerForMain(mainPath) {
        return new Promise((resolve) => {
            try {
                const cwd = path.dirname(mainPath);
                analyzerOutput.appendLine(`Running analyzer for ${mainPath}`);
                // prefer execFile for args array
                const args = [cwd, '--analyzer', '--all', '--json'];
                const child = cp.spawn('phlow', args, { cwd: cwd });
                let stdout = '';
                let stderr = '';
                child.stdout.on('data', d => { stdout += d.toString(); });
                child.stderr.on('data', d => { stderr += d.toString(); });
                child.on('error', err => {
                    analyzerOutput.appendLine(`Failed to spawn phlow for ${mainPath}: ${err.message}`);
                    projects.set(mainPath, { error: err.message, updatedAt: Date.now() });
                    resolve();
                });
                child.on('close', code => {
                    if (stderr && stderr.trim().length > 0) analyzerOutput.appendLine(`phlow stderr (${mainPath}): ${stderr}`);
                    if (code !== 0) {
                        analyzerOutput.appendLine(`phlow exited with code ${code} for ${mainPath}`);
                        projects.set(mainPath, { error: `exit ${code}`, updatedAt: Date.now(), raw: stdout });
                        resolve();
                        return;
                    }
                    try {
                        const parsed = stdout && stdout.trim() ? JSON.parse(stdout) : {};
                        projects.set(mainPath, { data: parsed, updatedAt: Date.now() });
                        analyzerOutput.appendLine(`Analyzer finished for ${mainPath}`);
                    } catch (e) {
                        analyzerOutput.appendLine(`Failed to parse phlow JSON for ${mainPath}: ${e.message}`);
                        projects.set(mainPath, { error: `json parse: ${e.message}`, raw: stdout, updatedAt: Date.now() });
                    }
                    resolve();
                });
            } catch (e) {
                analyzerOutput.appendLine(`Unexpected error running analyzer for ${mainPath}: ${e.message}`);
                projects.set(mainPath, { error: e.message, updatedAt: Date.now() });
                resolve();
            }
        });
    }

    async function runAllAnalyzers() {
        analyzerOutput.appendLine('Starting analyzer pass for all main.phlow files...');
        const mains = await findAllMainPhlow();
        if (!mains || mains.length === 0) {
            analyzerOutput.appendLine('No main.phlow files found in workspace.');
            return;
        }
        // run sequentially to avoid hammering the system; could parallelize if needed
        for (const m of mains) {
            await runAnalyzerForMain(m);
        }
        analyzerOutput.appendLine('Analyzer pass complete.');
        // rebuild module name index and refresh decorations
        try {
            rebuildModuleNames();
            updateAllOpenEditorsDecorations();
        } catch (e) {
            analyzerOutput.appendLine(`Error rebuilding modules: ${e.message}`);
        }
    }

    function rebuildModuleNames() {
        moduleNames = new Set();
        for (const [, v] of projects.entries()) {
            if (!v || !v.data || !Array.isArray(v.data.modules)) continue;
            for (const m of v.data.modules) {
                if (!m) continue;
                if (m.name) moduleNames.add(String(m.name));
                if (m.declared) {
                    const d = String(m.declared);
                    moduleNames.add(d);
                    // also add stripped './' variant
                    moduleNames.add(stripLeadingDotSlash(d));
                    // add basename for paths like ./inner
                    try {
                        const bn = path.basename(d);
                        if (bn) moduleNames.add(bn);
                    } catch (e) { /* ignore */ }
                }
            }
        }
        analyzerOutput.appendLine(`Indexed modules: ${Array.from(moduleNames).join(', ')}`);
    }

    function updateAllOpenEditorsDecorations() {
        for (const ed of vscode.window.visibleTextEditors) {
            updateDecorationsForEditor(ed);
        }
    }

    // Find all main.phlow files in workspace
    async function findAllMainPhlow() {
        try {
            const results = await vscode.workspace.findFiles('**/main.phlow', '**/node_modules/**');
            return results.map(u => u.fsPath);
        } catch (e) {
            analyzerOutput.appendLine(`Error finding main.phlow files: ${e.message}`);
            return [];
        }
    }

    // Find nearest main.phlow by walking up from a file/dir
    async function findNearestMainPhlow(fromPath) {
        const p = path.resolve(fromPath || '.');
        let stat = null;
        try { stat = await vscode.workspace.fs.stat(vscode.Uri.file(p)); } catch (e) { /* ignore */ }
        let current = stat && stat.type === vscode.FileType.Directory ? p : path.dirname(p);
        const rootFolders = (vscode.workspace.workspaceFolders || []).map(f => f.uri.fsPath);
        while (current) {
            const candidate = path.join(current, 'main.phlow');
            try {
                // use vscode.workspace.fs.stat to check existence
                await vscode.workspace.fs.stat(vscode.Uri.file(candidate));
                return candidate;
            } catch (e) {
                // not found, step up
            }
            const parent = path.dirname(current);
            if (parent === current) break;
            // stop if left the workspace roots
            if (rootFolders.length && !rootFolders.some(r => current.startsWith(r))) break;
            current = parent;
        }
        return null;
    }

    // Setup a global watcher for changes and re-run analyzers on changes (debounced)
    function setupWatcher() {
        // watch all files — if too broad, narrow to phlow files and project files
        const watcher = vscode.workspace.createFileSystemWatcher('**/*');
        context.subscriptions.push(watcher);
        const schedule = () => {
            if (changeDebounce) clearTimeout(changeDebounce);
            changeDebounce = setTimeout(() => {
                runAllAnalyzers().catch(e => analyzerOutput.appendLine(`runAllAnalyzers error: ${e.message}`));
            }, CHANGE_DEBOUNCE_MS);
        };
        watcher.onDidChange(schedule);
        watcher.onDidCreate(schedule);
        watcher.onDidDelete(schedule);
    }

    // Expose a command to manually refresh analyzers
    context.subscriptions.push(vscode.commands.registerCommand('phlow.refreshAnalyzer', async () => {
        await runAllAnalyzers();
        vscode.window.showInformationMessage('Phlow analyzer refreshed');
    }));

    // Start initial scan + watcher
    runAllAnalyzers().catch(e => analyzerOutput.appendLine(`Initial analyzer error: ${e.message}`));
    setupWatcher();

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
    // decoration for module names (from analyzer)
    let moduleDecoration = null;
    // set of module names collected from analyzer results
    let moduleNames = new Set();

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
        // ensure module decoration exists
        if (moduleDecoration) {
            try { moduleDecoration.dispose(); } catch (e) { /* ignore */ }
            moduleDecoration = null;
        }
        const modColor = vscode.workspace.getConfiguration('phlow').get('modules.color', '#C586C0');
        moduleDecoration = vscode.window.createTextEditorDecorationType({ color: modColor, fontWeight: 'bold' });
        context.subscriptions.push(moduleDecoration);
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
        // highlight module names where they appear
        try {
            const text = doc.getText();
            const lines = text.split(/\r?\n/);
            const modRanges = [];
            for (let i = 0; i < lines.length; i++) {
                const line = lines[i];
                // match list-item keys: '- logger:'
                const dashKeyMatch = line.match(/^\s*-\s+([A-Za-z0-9_\-\.\/]+)\s*:/);
                if (dashKeyMatch) {
                    const name = dashKeyMatch[1];
                    if (moduleNames.has(name) || moduleNames.has(stripLeadingDotSlash(name))) {
                        const startCol = line.indexOf(name);
                        const endCol = startCol + name.length;
                        modRanges.push(new vscode.Range(new vscode.Position(i, startCol), new vscode.Position(i, endCol)));
                        continue;
                    }
                }
                // match use: logger or - use: logger
                const useMatch = line.match(/^\s*(?:-\s*)?use\s*:\s*(?:"([^"]+)"|'([^']+)'|([^\s]+))/i);
                if (useMatch) {
                    const name = useMatch[1] || useMatch[2] || useMatch[3];
                    if (moduleNames.has(name) || moduleNames.has(stripLeadingDotSlash(name))) {
                        const idx = line.indexOf(name);
                        if (idx >= 0) {
                            modRanges.push(new vscode.Range(new vscode.Position(i, idx), new vscode.Position(i, idx + name.length)));
                        }
                    }
                }
            }
            if (moduleDecoration) editor.setDecorations(moduleDecoration, modRanges);
        } catch (e) {
            // ignore decoration errors
            analyzerOutput.appendLine(`module decoration error: ${e.message}`);
        }
    }

    function stripLeadingDotSlash(s) {
        if (!s) return s;
        return s.replace(/^\.\//, '');
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

