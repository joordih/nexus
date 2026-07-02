import * as vscode from 'vscode';
import * as cp from 'child_process';
import * as os from 'os';
import * as path from 'path';
import * as fs from 'fs';
import {
    LanguageClient,
    LanguageClientOptions,
    ServerOptions,
    State,
    Trace,
    TransportKind,
} from 'vscode-languageclient/node';

const LANGUAGE_ID = 'nexus';
const OUTPUT_CHANNEL_NAME = 'Nexus Language Server';
const TRACE_CHANNEL_NAME = 'Nexus Language Server Trace';

const DIAG_WITH_LOCATION = /^(error[^:]+)\s+en\s+(\d+):(\d+):\s+(.+)$/;
const DIAG_NO_LOCATION   = /^(funcion\s+\S+\s+no\s+siempre\s+retorna|metodo\s+\S+\s+no\s+siempre\s+retorna|tipo\s+incompatible|tipo\s+de\s+retorno\s+incompatible|nombre\s+no\s+resuelto:\s*.+)$/;

let languageClient: LanguageClient | undefined;
let nexusOutputChannel: vscode.OutputChannel;
let nexusTraceChannel: vscode.OutputChannel;

function logInfo(message: string): void {
    const stamp = new Date().toLocaleTimeString();
    nexusOutputChannel.appendLine(`[INFO  ${stamp}] ${message}`);
}

function logWarn(message: string): void {
    const stamp = new Date().toLocaleTimeString();
    nexusOutputChannel.appendLine(`[WARN  ${stamp}] ${message}`);
}

function logError(message: string): void {
    const stamp = new Date().toLocaleTimeString();
    nexusOutputChannel.appendLine(`[ERROR ${stamp}] ${message}`);
}

function relativePath(uri: vscode.Uri): string {
    const root = getWorkspaceRoot();
    const file = uri.fsPath;
    if (file.startsWith(root)) {
        return file.substring(root.length).replace(/^[/\\]/, '');
    }
    return file;
}

function traceLevelFromSetting(value: string): Trace {
    if (value === 'verbose') return Trace.Verbose;
    if (value === 'messages') return Trace.Messages;
    return Trace.Off;
}

function applyServerTrace(): void {
    if (!languageClient) return;
    const cfg = vscode.workspace.getConfiguration('nexus');
    const level = traceLevelFromSetting(cfg.get<string>('trace.server', 'off'));
    void languageClient.setTrace(level);
}

function resolveLspPath(configured: string, extensionPath: string): string | null {
    if (configured !== 'nexus-lsp') {
        return fs.existsSync(configured) ? configured : null;
    }
    const candidates: string[] = [];
    candidates.push(path.join(extensionPath, 'bin', 'nexus-lsp'));
    candidates.push(path.join(extensionPath, 'bin', 'nexus-lsp.exe'));
    const folders = vscode.workspace.workspaceFolders;
    if (folders && folders.length > 0) {
        candidates.push(path.join(folders[0].uri.fsPath, 'build', 'nexus-lsp'));
        candidates.push(path.join(folders[0].uri.fsPath, 'build', 'nexus-lsp.exe'));
    }
    candidates.push('nexus-lsp');
    candidates.push('nexus-lsp.exe');
    for (const c of candidates) {
        if (c === 'nexus-lsp' || c === 'nexus-lsp.exe') {
            try {
                const which = process.platform === 'win32' ? 'where' : 'which';
                cp.execSync(`${which} ${c}`, { stdio: 'ignore' });
                return c;
            } catch { continue; }
        }
        if (fs.existsSync(c)) return c;
    }
    return null;
}

function startLanguageClient(context: vscode.ExtensionContext): boolean {
    const cfg = vscode.workspace.getConfiguration('nexus');
    const lspPath = resolveLspPath(cfg.get<string>('lspPath', 'nexus-lsp'), context.extensionPath);
    if (!lspPath) {
        logWarn('Language server not found. Falling back to regex completion and compiler diagnostics.');
        return false;
    }

    const workspaceRoot = getWorkspaceRoot();
    logInfo(`Starting language server: ${lspPath}`);
    logInfo(`Workspace root: ${workspaceRoot}`);

    const serverOptions: ServerOptions = {
        run: {
            command: lspPath,
            args: [],
            transport: TransportKind.stdio,
            options: { cwd: workspaceRoot },
        },
        debug: {
            command: lspPath,
            args: [],
            transport: TransportKind.stdio,
            options: { cwd: workspaceRoot },
        },
    };

    const clientOptions: LanguageClientOptions = {
        documentSelector: [{ language: LANGUAGE_ID }],
        outputChannel: nexusOutputChannel,
        traceOutputChannel: nexusTraceChannel,
        synchronize: {
            fileEvents: vscode.workspace.createFileSystemWatcher('**/*.nx'),
        },
        middleware: {
            handleDiagnostics: (uri, diagnostics, next) => {
                const rel = relativePath(uri);
                if (diagnostics.length === 0) {
                    logInfo(`Analysis ${rel}: no issues`);
                } else {
                    const errors = diagnostics.filter(d => d.severity === vscode.DiagnosticSeverity.Error).length;
                    const warnings = diagnostics.filter(d => d.severity === vscode.DiagnosticSeverity.Warning).length;
                    logInfo(`Analysis ${rel}: ${errors} error(s), ${warnings} warning(s)`);
                    for (const diag of diagnostics) {
                        const sev = diag.severity === vscode.DiagnosticSeverity.Error ? 'error'
                            : diag.severity === vscode.DiagnosticSeverity.Warning ? 'warning'
                            : 'info';
                        const line = diag.range.start.line + 1;
                        const col = diag.range.start.character + 1;
                        nexusOutputChannel.appendLine(`  ${sev} at ${line}:${col}: ${diag.message}`);
                    }
                }
                next(uri, diagnostics);
            },
        },
    };

    languageClient = new LanguageClient(
        'nexusLsp',
        OUTPUT_CHANNEL_NAME,
        serverOptions,
        clientOptions,
    );

    languageClient.onDidChangeState(event => {
        if (event.newState === State.Starting) {
            logInfo('Language server state: starting');
        } else if (event.newState === State.Running) {
            logInfo('Language server state: running');
            applyServerTrace();
        } else if (event.newState === State.Stopped) {
            logInfo('Language server state: stopped');
        }
    });

    context.subscriptions.push(languageClient);
    context.subscriptions.push(
        vscode.workspace.onDidChangeConfiguration(event => {
            if (event.affectsConfiguration('nexus.trace.server')) {
                applyServerTrace();
            }
        }),
    );

    applyServerTrace();
    void languageClient.start().then(() => {
        logInfo('Language server started');
    });
    return true;
}

function getWorkspaceRoot(): string {
    const cfg = vscode.workspace.getConfiguration('nexus');
    if (cfg.get<string>('compilerWorkingDir', '')) {
        return cfg.get<string>('compilerWorkingDir', '');
    }
    const folders = vscode.workspace.workspaceFolders;
    if (folders && folders.length > 0) return folders[0].uri.fsPath;
    return process.cwd();
}

export function activate(context: vscode.ExtensionContext) {
    nexusOutputChannel = vscode.window.createOutputChannel(OUTPUT_CHANNEL_NAME);
    nexusTraceChannel = vscode.window.createOutputChannel(TRACE_CHANNEL_NAME);
    context.subscriptions.push(nexusOutputChannel, nexusTraceChannel);

    context.subscriptions.push(
        vscode.commands.registerCommand('nexus.showOutput', () => {
            nexusOutputChannel.show(true);
        }),
    );

    logInfo('Nexus extension activated');

    const lspActive = startLanguageClient(context);

    if (!lspActive) {
        activateRegexFallback(context);
    } else {
        activateCompilerDiagnostics(context);
    }
}

function activateCompilerDiagnostics(context: vscode.ExtensionContext) {
    const collection = vscode.languages.createDiagnosticCollection(LANGUAGE_ID);
    context.subscriptions.push(collection);

    const validate = (doc: vscode.TextDocument) => {
        if (doc.languageId !== LANGUAGE_ID) return;
        const cfg = vscode.workspace.getConfiguration('nexus');
        if (!cfg.get<boolean>('validateOnSave', true)) {
            collection.delete(doc.uri);
            return;
        }
        runCompilerCheck(doc, collection);
    };

    context.subscriptions.push(
        vscode.workspace.onDidSaveTextDocument(doc => validate(doc)),
    );
}

function activateRegexFallback(context: vscode.ExtensionContext) {
    wsIndex.rebuild();

    const collection = vscode.languages.createDiagnosticCollection(LANGUAGE_ID);
    context.subscriptions.push(collection);

    context.subscriptions.push(
        vscode.workspace.onDidSaveTextDocument(doc => {
            if (doc.languageId !== LANGUAGE_ID) return;
            wsIndex.indexText(doc.getText(), true);
            runCompilerCheck(doc, collection);
        }),
    );
    context.subscriptions.push(
        vscode.workspace.onDidOpenTextDocument(doc => {
            if (doc.languageId === LANGUAGE_ID) runCompilerCheck(doc, collection);
        }),
    );
    context.subscriptions.push(
        vscode.workspace.onDidCloseTextDocument(doc => collection.delete(doc.uri)),
    );
    vscode.workspace.textDocuments.forEach(doc => {
        if (doc.languageId === LANGUAGE_ID) runCompilerCheck(doc, collection);
    });

    context.subscriptions.push(
        vscode.languages.registerCompletionItemProvider(
            { language: LANGUAGE_ID },
            new NexusCompletionProvider(),
            '.',
        ),
    );
}

const METHOD_DEF  = /^[ \t]+([a-zA-Z_][a-zA-Z0-9_]*)\s*\(([^)]*)\)\s*(?::\s*([A-Za-z_][A-Za-z0-9_<>, ?]*))?\s*\{/gm;
const TOP_FN_DEF  = /^(?!(?:fun\s+[A-Z][a-zA-Z0-9_]*\.))([a-zA-Z_][a-zA-Z0-9_]*)\s*\(([^)]*)\)\s*(?::\s*([A-Za-z_][A-Za-z0-9_<>, ?]*))?\s*\{/gm;
const EXT_FN_DEF  = /^fun\s+([A-Z][a-zA-Z0-9_]*)\.([a-zA-Z_][a-zA-Z0-9_]*)\s*\(([^)]*)\)\s*(?::\s*([A-Za-z_][A-Za-z0-9_<>, ?]*))?\s*\{/gm;
const CLASS_START = /\b(?:class|data|value)\s+([A-Z][a-zA-Z0-9_]*)/g;
const MOD_DECL    = /^\s*module\s+(?:[a-zA-Z_][a-zA-Z0-9_.]*\.)?([a-zA-Z_][a-zA-Z0-9_]*)\s*$/m;

class WorkspaceIndex {
    private classMap     = new Map<string, vscode.CompletionItem[]>();
    private moduleMap    = new Map<string, vscode.CompletionItem[]>();
    private extensionMap = new Map<string, vscode.CompletionItem[]>();

    rebuild(): void {
        this.classMap.clear();
        this.moduleMap.clear();
        this.extensionMap.clear();
        for (const folder of vscode.workspace.workspaceFolders ?? []) {
            this.walkDir(folder.uri.fsPath);
        }
    }

    overlayDocument(doc: vscode.TextDocument): void {
        this.indexText(doc.getText(), true);
    }

    private walkDir(dir: string): void {
        try {
            for (const ent of fs.readdirSync(dir, { withFileTypes: true })) {
                const full = path.join(dir, ent.name);
                if (ent.isDirectory() && ent.name !== 'node_modules' && !ent.name.startsWith('.')) {
                    this.walkDir(full);
                } else if (ent.isFile() && ent.name.endsWith('.nx')) {
                    try { this.indexText(fs.readFileSync(full, 'utf8'), false); } catch {}
                }
            }
        } catch {}
    }

    indexText(text: string, overwrite: boolean): void {
        const modMatch = MOD_DECL.exec(text);
        const moduleName = modMatch?.[1] ?? null;

        CLASS_START.lastIndex = 0;
        let cm: RegExpExecArray | null;
        while ((cm = CLASS_START.exec(text)) !== null) {
            const className = cm[1];
            if (!overwrite && this.classMap.has(className)) continue;
            const methods = this.extractClassMethods(text, cm.index + cm[0].length, className);
            if (methods.length > 0) this.classMap.set(className, methods);
        }

        if (moduleName) {
            if (overwrite || !this.moduleMap.has(moduleName)) {
                const fns = this.extractTopLevelFunctions(text);
                if (fns.length > 0) this.moduleMap.set(moduleName, fns);
            }
        }

        EXT_FN_DEF.lastIndex = 0;
        let em: RegExpExecArray | null;
        while ((em = EXT_FN_DEF.exec(text)) !== null) {
            const typeName = em[1];
            const methodName = em[2];
            if (RESERVED.has(methodName)) continue;
            const params = em[3].trim();
            const ret = em[4]?.trim() ?? 'Void';
            const existing = this.extensionMap.get(typeName) ?? [];
            if (!existing.some(item => item.label === methodName)) {
                existing.push(makeMethod(methodName, params, ret, typeName));
                this.extensionMap.set(typeName, existing);
            }
        }
    }

    private extractClassMethods(text: string, from: number, className: string): vscode.CompletionItem[] {
        let i = from;
        while (i < text.length && text[i] !== '{') i++;
        if (i >= text.length) return [];

        let depth = 0, bodyEnd = i;
        for (let j = i; j < text.length; j++) {
            if (text[j] === '{') depth++;
            else if (text[j] === '}') { depth--; if (depth === 0) { bodyEnd = j; break; } }
        }
        return this.parseMethods(text.slice(i, bodyEnd), className);
    }

    private parseMethods(body: string, className: string): vscode.CompletionItem[] {
        const items: vscode.CompletionItem[] = [];
        const seen = new Set<string>();
        METHOD_DEF.lastIndex = 0;
        let m: RegExpExecArray | null;
        while ((m = METHOD_DEF.exec(body)) !== null) {
            const name = m[1];
            if (RESERVED.has(name) || seen.has(name)) continue;
            seen.add(name);
            items.push(makeMethod(name, m[2].trim(), m[3]?.trim() ?? 'Void', className));
        }
        return items;
    }

    private extractTopLevelFunctions(text: string): vscode.CompletionItem[] {
        const items: vscode.CompletionItem[] = [];
        const seen = new Set<string>();
        TOP_FN_DEF.lastIndex = 0;
        let m: RegExpExecArray | null;
        while ((m = TOP_FN_DEF.exec(text)) !== null) {
            const name = m[1];
            if (RESERVED.has(name) || seen.has(name)) continue;
            seen.add(name);
            const params = m[2].trim();
            const ret    = m[3]?.trim() ?? 'Void';
            const item   = new vscode.CompletionItem(name, vscode.CompletionItemKind.Function);
            item.detail  = `(${params}): ${ret}`;
            item.insertText = new vscode.SnippetString(params ? `${name}($0)` : `${name}()`);
            items.push(item);
        }
        return items;
    }

    methodsForClass(className: string): vscode.CompletionItem[] {
        return this.classMap.get(className) ?? [];
    }

    methodsForModule(modName: string): vscode.CompletionItem[] {
        return this.moduleMap.get(modName) ?? [];
    }

    hasClass(name: string): boolean { return this.classMap.has(name); }
    hasModule(name: string): boolean { return this.moduleMap.has(name); }

    extensionsForType(typeName: string): vscode.CompletionItem[] {
        return this.extensionMap.get(typeName) ?? [];
    }
}

const wsIndex = new WorkspaceIndex();

function makeMethod(name: string, params: string, ret: string, owner: string): vscode.CompletionItem {
    const item = new vscode.CompletionItem(name, vscode.CompletionItemKind.Method);
    item.detail = `(${params}): ${ret}`;
    item.insertText = new vscode.SnippetString(params ? `${name}($0)` : `${name}()`);
    item.documentation = new vscode.MarkdownString(`Method of \`${owner}\``);
    return item;
}

class NexusCompletionProvider implements vscode.CompletionItemProvider {
    provideCompletionItems(
        document: vscode.TextDocument,
        position: vscode.Position,
        _token: vscode.CancellationToken,
        context: vscode.CompletionContext,
    ): vscode.CompletionItem[] {
        wsIndex.overlayDocument(document);

        const lineText   = document.lineAt(position).text;
        const textBefore = lineText.substring(0, position.character);

        if (context.triggerCharacter === '.' || /\.\s*$/.test(textBefore)) {
            return memberCompletions(textBefore, document, position);
        }

        if (/^\s*import\s/.test(lineText)) {
            return importCompletions();
        }

        return [
            ...keywordCompletions(),
            ...typeCompletions(),
            ...constantCompletions(),
            ...globalFunctionCompletions(),
            ...documentSymbols(document, position),
        ];
    }
}

function memberCompletions(
    textBefore: string,
    document: vscode.TextDocument,
    position: vscode.Position,
): vscode.CompletionItem[] {
    const receiverName = /([a-zA-Z_][a-zA-Z0-9_]*)\s*\.\s*$/.exec(textBefore)?.[1];
    if (!receiverName) return fallbackMembers();

    if (receiverName === 'io') return ioMethods();
    if (wsIndex.hasModule(receiverName)) return wsIndex.methodsForModule(receiverName);

    const inferredType = inferType(receiverName, document, position);
    if (inferredType) {
        return builtinMethodsForType(inferredType);
    }

    return fallbackMembers();
}

function builtinMethodsForType(typeName: string): vscode.CompletionItem[] {
    const builtins: vscode.CompletionItem[] = [];
    if (typeName === 'String') builtins.push(...stringMethods());
    else if (typeName === 'List') builtins.push(...listMethods());
    else if (typeName === 'Map') builtins.push(...mapMethods());
    else if (typeName === 'Int' || typeName === 'Long') builtins.push(...intMethods());
    else if (typeName === 'Float' || typeName === 'Double') builtins.push(...floatMethods());
    else if (typeName === 'Bool' || typeName === 'Char') builtins.push(...primitiveToStringMethods());

    const extensions = wsIndex.extensionsForType(typeName);
    const classMethods = wsIndex.hasClass(typeName) ? wsIndex.methodsForClass(typeName) : [];

    const merged = [...builtins, ...extensions, ...classMethods];
    if (merged.length > 0) return merged;
    return fallbackMembers();
}

function fallbackMembers(): vscode.CompletionItem[] {
    return [...stringMethods(), ...listMethods(), ...mapMethods(), ...intMethods()];
}

function inferType(name: string, doc: vscode.TextDocument, pos: vscode.Position): string | null {
    const text = doc.getText(new vscode.Range(new vscode.Position(0, 0), pos));
    const esc  = escapeRe(name);
    let m: RegExpExecArray | null;
    let lastType: string | null = null;

    const declRe = new RegExp(`\\b(?:var|val|final)\\s+${esc}\\s*:\\s*([A-Za-z_][A-Za-z0-9_<>, ?]*)`, 'g');
    while ((m = declRe.exec(text)) !== null) lastType = m[1].trim().split('<')[0].trim().replace(/\?$/, '');
    if (lastType) return lastType;

    const ctorRe = new RegExp(`\\b(?:var|val|final)\\s+${esc}\\s*=\\s*([A-Z][A-Za-z0-9_]*)\\s*\\(`, 'g');
    while ((m = ctorRe.exec(text)) !== null) lastType = m[1].trim();
    if (lastType) return lastType;

    const strLitRe = new RegExp(`\\b(?:var|val|final)\\s+${esc}\\s*=\\s*r?"`, 'g');
    if (strLitRe.test(text)) return 'String';

    const floatLitRe = new RegExp(`\\b(?:var|val|final)\\s+${esc}\\s*=\\s*[0-9]+\\.[0-9]+`, 'g');
    if (floatLitRe.test(text)) return 'Float';

    const paramRe = new RegExp(`\\b${esc}\\s*:\\s*([A-Za-z_][A-Za-z0-9_<>, ?]*)`, 'g');
    while ((m = paramRe.exec(text)) !== null) lastType = m[1].trim().split('<')[0].trim().replace(/\?$/, '');
    return lastType;
}

function escapeRe(s: string): string {
    return s.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

function method(label: string, detail: string, doc: string, snippet: string): vscode.CompletionItem {
    const item = new vscode.CompletionItem(label, vscode.CompletionItemKind.Method);
    item.detail = detail;
    item.documentation = new vscode.MarkdownString(doc);
    item.insertText = new vscode.SnippetString(snippet);
    return item;
}

function ioMethods(): vscode.CompletionItem[] {
    return [
        method('println(value)', 'io -> Void', 'Prints a value followed by a newline.', 'println(${1:value})'),
        method('print(value)', 'io -> Void', 'Prints a value without a newline.', 'print(${1:value})'),
        method('readLine()', 'io -> String', 'Reads a line from standard input.', 'readLine()'),
        method('readFile(path)', 'io -> String', 'Reads the entire contents of a file.', 'readFile(${1:path})'),
        method('writeFile(path, content)', 'io -> Void', 'Writes content to a file.', 'writeFile(${1:path}, ${2:content})'),
    ];
}

function stringMethods(): vscode.CompletionItem[] {
    return [
        method('length()', 'String -> Int', 'Returns the number of characters.', 'length()'),
        method('charAt(i)', 'String -> Char', 'Returns the character at index i.', 'charAt(${1:i})'),
        method('substring(start, end)', 'String -> String', 'Returns a substring.', 'substring(${1:start}, ${2:end})'),
        method('contains(sub)', 'String -> Bool', 'Returns true if this string contains sub.', 'contains(${1:sub})'),
        method('startsWith(prefix)', 'String -> Bool', 'Returns true if this string starts with prefix.', 'startsWith(${1:prefix})'),
        method('endsWith(suffix)', 'String -> Bool', 'Returns true if this string ends with suffix.', 'endsWith(${1:suffix})'),
        method('toInt()', 'String -> Int', 'Parses the string as an integer.', 'toInt()'),
        method('toString()', 'String -> String', 'Returns the string itself.', 'toString()'),
    ];
}

function listMethods(): vscode.CompletionItem[] {
    return [
        method('len()', 'List -> Int', 'Returns the number of elements.', 'len()'),
        method('get(i)', 'List -> T', 'Returns the element at index i.', 'get(${1:i})'),
        method('set(i, value)', 'List -> Void', 'Sets the element at index i.', 'set(${1:i}, ${2:value})'),
        method('push(value)', 'List -> Void', 'Appends value to the end of the list.', 'push(${1:value})'),
    ];
}

function mapMethods(): vscode.CompletionItem[] {
    return [
        method('contains(key)', 'Map -> Bool', 'Returns true if the map has the given key.', 'contains(${1:key})'),
        method('get(key)', 'Map -> V', 'Returns the value for key.', 'get(${1:key})'),
        method('put(key, value)', 'Map -> Void', 'Inserts or updates the entry for key.', 'put(${1:key}, ${2:value})'),
    ];
}

function intMethods(): vscode.CompletionItem[] {
    return [
        method('toString()', 'Int -> String', 'Converts the integer to its string representation.', 'toString()'),
    ];
}

function floatMethods(): vscode.CompletionItem[] {
    return [
        method('toString()', 'Float/Double -> String', 'Converts the floating-point value to a string.', 'toString()'),
    ];
}

function primitiveToStringMethods(): vscode.CompletionItem[] {
    return [
        method('toString()', '-> String', 'Converts the value to its string representation.', 'toString()'),
    ];
}

function importCompletions(): vscode.CompletionItem[] {
    return [
        fn('import path', 'import ${1:std.io}', '(path) -> Void', 'Import a module by dot path.'),
        fn('import alias', 'import ${1:std.network.http_client} as ${2:http}', '(path as alias) -> Void', 'Import a module under a local alias.'),
        fn('import group', 'import ${1:std.core}.{${2:math, strings}}', '(grouped) -> Void', 'Import several modules from the same package.'),
        fn('import wildcard', 'import ${1:std.core}.*', '(wildcard) -> Void', 'Import every module in a package directory.'),
    ];
}

function kw(label: string, snippet: string, detail: string): vscode.CompletionItem {
    const item = new vscode.CompletionItem(label, vscode.CompletionItemKind.Keyword);
    item.insertText = new vscode.SnippetString(snippet);
    item.detail = detail;
    item.sortText = '0_' + label;
    return item;
}

function keywordCompletions(): vscode.CompletionItem[] {
    return [
        kw('if', 'if ($1) {\n\t$0\n}', 'if statement'),
        kw('while', 'while ($1) {\n\t$0\n}', 'while loop'),
        kw('for', 'for ($1 in $2) {\n\t$0\n}', 'for-in loop'),
        kw('return', 'return $0', 'return statement'),
        kw('var', 'var $1: $2 = $0', 'mutable variable'),
        kw('val', 'val $1 = $0', 'immutable local binding'),
        kw('final', 'final $1: $2 = $0', 'immutable global'),
        kw('data', 'data $1 {\n\t$0\n}', 'data class'),
        kw('class', 'class $1 {\n$0\n}', 'class definition'),
        kw('fun', 'fun ${1:Type}.${2:method}(${3:params}): ${4:Return} {\n\t$0\n}', 'extension function'),
        kw('import', 'import ${1:std.io}', 'import statement'),
        kw('try', 'try {\n\t$1\n} catch (${2:err}) {\n\t$0\n}', 'try/catch block'),
        kw('throw', 'throw', 'throw statement'),
    ];
}

function ty(label: string, snippet: string, doc: string): vscode.CompletionItem {
    const item = new vscode.CompletionItem(label, vscode.CompletionItemKind.Class);
    item.insertText = new vscode.SnippetString(snippet);
    item.documentation = new vscode.MarkdownString(doc);
    item.sortText = '1_' + label;
    return item;
}

function typeCompletions(): vscode.CompletionItem[] {
    return [
        ty('Int', 'Int', 'A 64-bit integer.'),
        ty('Long', 'Long', 'A 64-bit integer.'),
        ty('Float', 'Float', 'A single-precision float.'),
        ty('Double', 'Double', 'A double-precision float.'),
        ty('Char', 'Char', 'A Unicode code unit.'),
        ty('String', 'String', 'An immutable string.'),
        ty('Bool', 'Bool', 'A boolean value.'),
        ty('List', 'List<${1:T}>', 'A mutable ordered list.'),
        ty('Map', 'Map<${1:K}, ${2:V}>', 'A hash map.'),
        ty('Void', 'Void', 'No return value.'),
    ];
}

function constantCompletions(): vscode.CompletionItem[] {
    const items: vscode.CompletionItem[] = [];
    for (const [label, detail] of [['true', 'Bool'], ['false', 'Bool'], ['null', 'Null']] as [string, string][]) {
        const item = new vscode.CompletionItem(label, vscode.CompletionItemKind.Constant);
        item.detail = detail;
        item.sortText = '2_' + label;
        items.push(item);
    }
    return items;
}

function fn(label: string, snippet: string, detail: string, doc: string): vscode.CompletionItem {
    const item = new vscode.CompletionItem(label, vscode.CompletionItemKind.Function);
    item.insertText = new vscode.SnippetString(snippet);
    item.detail = detail;
    item.documentation = new vscode.MarkdownString(doc);
    item.sortText = '3_' + label;
    return item;
}

function globalFunctionCompletions(): vscode.CompletionItem[] {
    return [
        fn('println', 'println($0)', '(value) -> Void', 'Prints a value followed by a newline.'),
        fn('readLine', 'readLine()', '() -> String', 'Reads a line from standard input.'),
        fn('readFile', 'readFile($1)', '(path: String) -> String', 'Reads the entire contents of a file.'),
    ];
}

interface DocSymbol { name: string; kind: vscode.CompletionItemKind; detail: string; }

function documentSymbols(document: vscode.TextDocument, currentPos: vscode.Position): vscode.CompletionItem[] {
    const text   = document.getText();
    const before = document.getText(new vscode.Range(new vscode.Position(0, 0), currentPos));
    const symbols = new Map<string, DocSymbol>();

    const FN_DEF = /^[ \t]*(?!(?:fun\s+[A-Z][a-zA-Z0-9_]*\.))([a-zA-Z_][a-zA-Z0-9_]*)\s*\(([^)]*)\)\s*(?::\s*([A-Za-z_][A-Za-z0-9_<>, ?]*))?\s*\{/gm;
    for (const m of text.matchAll(FN_DEF)) {
        const name = m[1];
        if (RESERVED.has(name)) continue;
        symbols.set(name, { name, kind: vscode.CompletionItemKind.Function, detail: `(${m[2].trim()}): ${m[3]?.trim() ?? 'Void'}` });
    }

    const EXT_DEF = /^fun\s+([A-Z][a-zA-Z0-9_]*)\.([a-zA-Z_][a-zA-Z0-9_]*)\s*\(([^)]*)\)\s*(?::\s*([A-Za-z_][A-Za-z0-9_<>, ?]*))?\s*\{/gm;
    for (const m of text.matchAll(EXT_DEF)) {
        const label = `${m[1]}.${m[2]}`;
        symbols.set(label, {
            name: label,
            kind: vscode.CompletionItemKind.Method,
            detail: `(${m[3].trim()}): ${m[4]?.trim() ?? 'Void'}`,
        });
    }

    const CLASS_DEF = /\b(?:class|data|value|interface)\s+([A-Z][a-zA-Z0-9_]*)/g;
    for (const m of text.matchAll(CLASS_DEF)) {
        symbols.set(m[1], { name: m[1], kind: vscode.CompletionItemKind.Class, detail: 'class' });
    }

    const VAR_DECL = /\b(?:var|val|final)\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*(?::\s*([a-zA-Z_][a-zA-Z0-9_<>, ?]*))?/g;
    for (const m of before.matchAll(VAR_DECL)) {
        if (!symbols.has(m[1])) {
            const detail = m[2]?.trim() ?? 'inferred';
            symbols.set(m[1], { name: m[1], kind: vscode.CompletionItemKind.Variable, detail });
        }
    }

    return [...symbols.values()].map(s => {
        const item = new vscode.CompletionItem(s.name, s.kind);
        item.detail   = s.detail;
        item.sortText = '4_' + s.name;
        return item;
    });
}

const RESERVED = new Set([
    'if','else','while','for','in','return','break','continue','try','catch','throw',
    'var','val','final','fun','class','data','value','interface','annotation',
    'import','module','extends','implements','this','switch','case','default',
    'true','false','null',
]);

function resolveCompilerPath(configured: string): string {
    if (configured !== 'nxc') return configured;
    if (process.platform === 'win32') {
        try { cp.execSync('where nxc.exe', { stdio: 'ignore' }); return 'nxc.exe'; } catch {}
    }
    return 'nxc';
}

function runCompilerCheck(doc: vscode.TextDocument, collection: vscode.DiagnosticCollection) {
    const cfg = vscode.workspace.getConfiguration('nexus');
    if (!cfg.get<boolean>('validateOnSave', true)) {
        collection.set(doc.uri, []);
        return;
    }

    const compilerPath = resolveCompilerPath(cfg.get<string>('compilerPath', 'nxc'));
    const tmpOut = path.join(os.tmpdir(), `nxc_check_${Date.now()}`);
    const cwd    = getWorkspaceRoot();
    const args   = ['compile', doc.fileName, tmpOut];

    let stdout = '', stderr = '';
    const proc = cp.spawn(compilerPath, args, { cwd });
    proc.stdout.on('data', (data: Buffer) => { stdout += data.toString(); });
    proc.stderr.on('data', (data: Buffer) => { stderr += data.toString(); });

    proc.on('error', (err: NodeJS.ErrnoException) => {
        if (err.code === 'ENOENT') {
            logError(`Compiler not found at '${compilerPath}'`);
            vscode.window.showErrorMessage(
                `Nexus: compiler not found at '${compilerPath}'. Set nexus.compilerPath in settings.`,
            );
        }
        collection.set(doc.uri, []);
    });

    proc.on('close', code => {
        try { fs.unlinkSync(tmpOut); } catch {}
        try { fs.unlinkSync(tmpOut + '.c'); } catch {}
        try { fs.unlinkSync(tmpOut + '.exe'); } catch {}
        const diags = parseDiagnostics(stdout + '\n' + stderr, doc);
        const rel = relativePath(doc.uri);
        if (diags.length === 0) {
            logInfo(`Compiler check ${rel}: no issues (exit ${code ?? 0})`);
        } else {
            logInfo(`Compiler check ${rel}: ${diags.length} issue(s) (exit ${code ?? 0})`);
            for (const diag of diags) {
                const line = diag.range.start.line + 1;
                const col = diag.range.start.character + 1;
                nexusOutputChannel.appendLine(`  error at ${line}:${col}: ${diag.message}`);
            }
        }
        collection.set(doc.uri, diags);
    });
}

function parseDiagnostics(output: string, doc: vscode.TextDocument): vscode.Diagnostic[] {
    const diagnostics: vscode.Diagnostic[] = [];
    for (const raw of output.split('\n')) {
        const line = raw.trim();
        if (!line) continue;
        if (line.startsWith('In file included') || line.startsWith('runtime') ||
            line.startsWith('fatal error:')     || line.includes('note:') ||
            line.includes('clang fallo')        || line.includes('compilacion C fallo')) continue;

        const withLoc = DIAG_WITH_LOCATION.exec(line);
        if (withLoc) {
            const lineNum = parseInt(withLoc[2], 10) - 1;
            const colNum  = Math.max(0, parseInt(withLoc[3], 10) - 1);
            const docLine = doc.lineAt(Math.min(lineNum, doc.lineCount - 1));
            const range   = new vscode.Range(
                Math.min(lineNum, doc.lineCount - 1), colNum,
                Math.min(lineNum, doc.lineCount - 1), docLine.text.length,
            );
            diagnostics.push(new vscode.Diagnostic(
                range, withLoc[4].trim(),
                line.includes('error') ? vscode.DiagnosticSeverity.Error : vscode.DiagnosticSeverity.Warning,
            ));
            continue;
        }
        const noLoc = DIAG_NO_LOCATION.exec(line);
        if (noLoc) {
            const range = new vscode.Range(0, 0, 0, doc.lineAt(0).text.length);
            diagnostics.push(new vscode.Diagnostic(range, line, vscode.DiagnosticSeverity.Error));
        }
    }
    return diagnostics;
}

export async function deactivate(): Promise<void> {
    if (languageClient) {
        logInfo('Stopping language server');
        await languageClient.stop();
    }
}