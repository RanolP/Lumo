import { createEffect, createSignal, For, onCleanup, onMount, Show } from "solid-js";
import * as monaco from "monaco-editor/esm/vs/editor/editor.api";
import "monaco-editor/esm/vs/basic-languages/typescript/typescript.contribution";
import editorWorker from "monaco-editor/esm/vs/editor/editor.worker?worker";
import "monaco-editor/esm/vs/language/typescript/monaco.contribution";
import tsWorker from "monaco-editor/esm/vs/language/typescript/ts.worker?worker";

declare global {
  interface Window {
    MonacoEnvironment?: {
      getWorker: (_moduleId: string, _label: string) => Worker;
    };
  }
}

window.MonacoEnvironment = {
  getWorker: (_moduleId, label) => {
    if (label === "typescript" || label === "javascript") {
      return new tsWorker();
    }
    return new editorWorker();
  },
};

const LANGUAGE_ID = "lumo";
const URI = "file:///main.lumo";
const OUT_TS_URI = "file:///out.ts";
const INITIAL_SOURCE = `use libstd.io.{IO};

fn main() {
  IO.println("Hello, World!")
}
`;

type WasmModule = typeof import("./wasm/lumo_playground_wasm.js");
type CompilerResult = ReturnType<InstanceType<WasmModule["WasmCompiler"]>["compile"]>;
type TsMarker = monaco.editor.IMarker;
type TsTextChange = { span: { start: number; length: number }; newText: string };

function runGeneratedJs(code: string): Promise<string[]> {
  return new Promise((resolve) => {
    const encodedUserCode = JSON.stringify(code);
    const workerSource = [
      `const __userCode = ${encodedUserCode};`,
      "const __logs = [];",
      "const __formatValue = (value) => {",
      "  if (typeof value === 'string') return value;",
      "  try { return JSON.stringify(value); } catch { return String(value); }",
      "};",
      "const __formatError = (cause) => {",
      "  if (cause instanceof Error) {",
      "    return cause.stack || `${cause.name}: ${cause.message}`;",
      "  }",
      "  try { return JSON.stringify(cause); } catch { return String(cause); }",
      "};",
      "const __consoleProxy = {",
      "  log: (...args) => { __logs.push(args.map(__formatValue).join(' ')); },",
      "  info: (...args) => { __logs.push(`[info] ${args.map(__formatValue).join(' ')}`); },",
      "  warn: (...args) => { __logs.push(`[warn] ${args.map(__formatValue).join(' ')}`); },",
      "  error: (...args) => { __logs.push(`[error] ${args.map(__formatValue).join(' ')}`); },",
      "};",
      "globalThis.console = __consoleProxy;",
      "(async () => {",
      "  const __userUrl = URL.createObjectURL(new Blob([__userCode], { type: 'text/javascript' }));",
      "  try {",
      "    const __mod = await import(__userUrl);",
      "    if (typeof __mod.main === 'function') {",
      "      if (__mod.main.length === 0) {",
      "        const __mainResult = await __mod.main();",
      "        if (__mainResult !== undefined) {",
      "          __logs.push(`[result] ${__formatValue(__mainResult)}`);",
      "        }",
      "      } else {",
      "        __logs.push(`[info] skipped auto-call: main expects ${__mod.main.length} argument(s)`);",
      "      }",
      "    }",
      "  } catch (cause) {",
      "    __logs.push(`[runtime error] ${__formatError(cause)}`);",
      "  } finally {",
      "    URL.revokeObjectURL(__userUrl);",
      "    if (__logs.length === 0) {",
      "      __logs.push('(program executed with no output)');",
      "    }",
      "    postMessage({ logs: __logs });",
      "  }",
      "})();",
    ].join("\n");

    const blob = new Blob([workerSource], { type: "text/javascript" });
    const url = URL.createObjectURL(blob);
    const worker = new Worker(url, { type: "module" });
    let settled = false;

    const cleanup = () => {
      worker.terminate();
      URL.revokeObjectURL(url);
    };

    const finish = (logs: string[]) => {
      if (settled) return;
      settled = true;
      cleanup();
      resolve(logs);
    };

    const timeout = setTimeout(() => {
      finish(["[runtime error] execution timed out"]);
    }, 1500);

    worker.onmessage = (event: MessageEvent<{ logs?: unknown }>) => {
      clearTimeout(timeout);
      const logs = Array.isArray(event.data?.logs)
        ? event.data.logs.map((item) => String(item))
        : ["(program executed with no output)"];
      finish(logs);
    };

    worker.onerror = (event: ErrorEvent) => {
      clearTimeout(timeout);
      const location =
        event.filename && (event.lineno || event.colno)
          ? ` (${event.filename}:${event.lineno}:${event.colno})`
          : "";
      const detail =
        event.error instanceof Error
          ? event.error.stack || `${event.error.name}: ${event.error.message}`
          : event.message;
      finish([`[runtime error] ${detail || "worker execution failed"}${location}`]);
    };

    worker.onmessageerror = () => {
      clearTimeout(timeout);
      finish(["[runtime error] worker message deserialization failed"]);
    };
  });
}

function sortMarkers(markers: readonly TsMarker[]): TsMarker[] {
  return [...markers].sort((left, right) => {
    if (left.severity !== right.severity) return right.severity - left.severity;
    if (left.startLineNumber !== right.startLineNumber) {
      return left.startLineNumber - right.startLineNumber;
    }
    if (left.startColumn !== right.startColumn) return left.startColumn - right.startColumn;
    return left.message.localeCompare(right.message);
  });
}

function formatMarkerSeverity(severity: monaco.MarkerSeverity): string {
  switch (severity) {
    case monaco.MarkerSeverity.Error:
      return "error";
    case monaco.MarkerSeverity.Warning:
      return "warning";
    case monaco.MarkerSeverity.Info:
      return "info";
    default:
      return "hint";
  }
}

function applyTextChanges(model: monaco.editor.ITextModel, changes: readonly TsTextChange[]) {
  if (changes.length === 0) return;

  model.pushEditOperations(
    [],
    changes.map((change) => {
      const start = model.getPositionAt(change.span.start);
      const end = model.getPositionAt(change.span.start + change.span.length);
      return {
        range: new monaco.Range(start.lineNumber, start.column, end.lineNumber, end.column),
        text: change.newText,
        forceMoveMarkers: true,
      };
    }),
    () => null,
  );
}

export default function App() {
  let sourceEditorRef: HTMLDivElement | undefined;
  let tsViewRef: HTMLDivElement | undefined;
  let sourceEditor: monaco.editor.IStandaloneCodeEditor | undefined;
  let tsViewEditor: monaco.editor.IStandaloneCodeEditor | undefined;

  const [ready, setReady] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const [emittedTs, setEmittedTs] = createSignal("");
  const [emittedJs, setEmittedJs] = createSignal("");
  const [typedAst, setTypedAst] = createSignal("");
  const [loweredTypedAst, setLoweredTypedAst] = createSignal("");
  const [activeRightTab, setActiveRightTab] = createSignal<"ts" | "typed-ast" | "lowered-ast">(
    "ts",
  );
  const [runtimeOutput, setRuntimeOutput] = createSignal<string[]>([]);
  const [problems, setProblems] = createSignal<CompilerResult["diagnostics"]>([]);
  const [tsProblems, setTsProblems] = createSignal<TsMarker[]>([]);

  onMount(async () => {
    try {
      const wasm = (await import(
        /* @vite-ignore */
        "./wasm/lumo_playground_wasm.js"
      )) as WasmModule;

      await wasm.default();
      const compiler = new wasm.WasmCompiler();
      const lsp = new wasm.WasmLsp();
      let nextRequestId = 0;
      let nextTsFormatRequest = 0;
      let currentText = INITIAL_SOURCE;
      let documentVersion = 1;

      monaco.languages.typescript.typescriptDefaults.setEagerModelSync(true);
      monaco.languages.typescript.typescriptDefaults.setDiagnosticsOptions({
        noSemanticValidation: false,
        noSyntaxValidation: false,
        noSuggestionDiagnostics: false,
      });
      monaco.languages.typescript.typescriptDefaults.setCompilerOptions({
        allowNonTsExtensions: true,
        module: monaco.languages.typescript.ModuleKind.ESNext,
        moduleResolution: monaco.languages.typescript.ModuleResolutionKind.NodeJs,
        target: monaco.languages.typescript.ScriptTarget.ES2020,
        lib: ["es2022", "dom"],
        strict: true,
        noEmit: true,
      });

      monaco.languages.register({ id: LANGUAGE_ID });

      monaco.languages.setMonarchTokensProvider(LANGUAGE_ID, {
        keywords: ["data", "fn", "extern", "type", "let", "in", "produce", "thunk", "force", "match", "use", "cap", "handle", "impl", "if", "else"],
        operators: ["#", "[", "]", "{", "}", "(", ")", ";", ":=", "=>", "=", ":", ",", ".", "*", "/", "+", "-", "<", ">"],
        tokenizer: {
          root: [
            [/#\s*\[/, { token: "operator", next: "@attribute" }],
            [/\"(?:\\.|[^\"\\])*\"/, "string"],
            [
              /\b[A-Za-z_][A-Za-z0-9_]*\b/,
              {
                cases: {
                  "@keywords": "keyword",
                  "@default": "identifier",
                },
              },
            ],
            [/#|\[|\]|\{|\}|\(|\)|;|\:=|=>|=|:|,|\.|\*|\//, "operator"],
            [/\s+/, "white"],
          ],
          attribute: [
            [/\]/, { token: "operator", next: "@pop" }],
            [/\"(?:\\.|[^\"\\])*\"/, "string"],
            [/\b[A-Za-z_][A-Za-z0-9_]*\b/, "identifier"],
            [/#|\[|\]|\{|\}|\(|\)|;|\:=|=>|=|:|,|\.|\*|\//, "operator"],
            [/\s+/, "white"],
          ],
        },
      });

      const invokeLsp = (message: unknown) => lsp.handle_message(JSON.stringify(message));

      invokeLsp({ jsonrpc: "2.0", id: 1, method: "initialize", params: {} });
      invokeLsp({ jsonrpc: "2.0", method: "initialized", params: {} });
      invokeLsp({
        jsonrpc: "2.0",
        method: "textDocument/didOpen",
        params: {
          textDocument: {
            uri: URI,
            languageId: LANGUAGE_ID,
            version: documentVersion,
            text: currentText,
          },
        },
      });

      const sourceModel = monaco.editor.createModel(
        currentText,
        LANGUAGE_ID,
        monaco.Uri.parse(URI),
      );
      const tsModel = monaco.editor.createModel("", "typescript", monaco.Uri.parse(OUT_TS_URI));
      const syncTsProblems = () => {
        setTsProblems(sortMarkers(monaco.editor.getModelMarkers({ resource: tsModel.uri })));
      };
      const formatTsModel = async (requestId: number, sourceText: string) => {
        if (!sourceText.trim()) {
          syncTsProblems();
          return;
        }

        try {
          const getWorker = await monaco.languages.typescript.getTypeScriptWorker();
          if (requestId !== nextTsFormatRequest || tsModel.isDisposed()) return;

          const worker = await getWorker(tsModel.uri);
          if (requestId !== nextTsFormatRequest || tsModel.isDisposed()) return;

          const edits = (await worker.getFormattingEditsForDocument(tsModel.uri.toString(), {
            ConvertTabsToSpaces: true,
            IndentSize: 2,
            TabSize: 2,
            NewLineCharacter: "\n",
            InsertSpaceAfterCommaDelimiter: true,
            InsertSpaceAfterSemicolonInForStatements: true,
            InsertSpaceBeforeAndAfterBinaryOperators: true,
            InsertSpaceAfterKeywordsInControlFlowStatements: true,
            InsertSpaceAfterFunctionKeywordForAnonymousFunctions: false,
            InsertSpaceAfterOpeningAndBeforeClosingNonemptyParenthesis: false,
            InsertSpaceAfterOpeningAndBeforeClosingNonemptyBrackets: false,
            InsertSpaceAfterOpeningAndBeforeClosingNonemptyBraces: true,
            InsertSpaceAfterOpeningAndBeforeClosingTemplateStringBraces: false,
            InsertSpaceAfterOpeningAndBeforeClosingJsxExpressionBraces: false,
            InsertSpaceAfterTypeAssertion: false,
            PlaceOpenBraceOnNewLineForFunctions: false,
            PlaceOpenBraceOnNewLineForControlBlocks: false,
            Semicolons: "insert",
          })) as TsTextChange[];

          if (requestId !== nextTsFormatRequest || tsModel.isDisposed()) return;
          if (tsModel.getValue() !== sourceText) return;

          applyTextChanges(tsModel, edits);
        } catch {
          // Keep the raw emitter output if Monaco's formatter isn't ready yet.
        } finally {
          syncTsProblems();
        }
      };

      const syncDocument = (text: string) => {
        documentVersion += 1;
        currentText = text;

        invokeLsp({
          jsonrpc: "2.0",
          method: "textDocument/didChange",
          params: {
            textDocument: { uri: URI, version: documentVersion },
            contentChanges: [{ text }],
          },
        });

        compiler.set_file(URI, text);
        const compileResult = compiler.compile(URI) as CompilerResult;
        const allDiagnostics = [...compileResult.parse_errors, ...compileResult.diagnostics];

        const rawEmittedTs = compileResult.emitted_ts || "";
        setEmittedTs(rawEmittedTs);
        setEmittedJs(compileResult.emitted_js);
        setTypedAst(compileResult.typed_ast || "");
        setLoweredTypedAst(compileResult.lowered_typed_ast || "");
        setProblems(allDiagnostics);
        tsModel.setValue(rawEmittedTs);
        syncTsProblems();
        nextTsFormatRequest += 1;
        void formatTsModel(nextTsFormatRequest, rawEmittedTs);

        monaco.editor.setModelMarkers(
          sourceModel,
          "lumo-compiler",
          allDiagnostics.map((diagnostic: CompilerResult["diagnostics"][number]) => ({
            message: diagnostic.message,
            severity: monaco.MarkerSeverity.Error,
            startLineNumber: diagnostic.start_line + 1,
            startColumn: diagnostic.start_character + 1,
            endLineNumber: diagnostic.end_line + 1,
            endColumn: diagnostic.end_character + 1,
          })),
        );
      };

      monaco.languages.registerDocumentSemanticTokensProvider(LANGUAGE_ID, {
        getLegend() {
          return {
            tokenTypes: ["keyword", "variable", "string", "operator"],
            tokenModifiers: [],
          };
        },
        provideDocumentSemanticTokens: (_model, lastResultId) => {
          const id = ++nextRequestId;
          const output = invokeLsp({
            jsonrpc: "2.0",
            id,
            method: lastResultId
              ? "textDocument/semanticTokens/full/delta"
              : "textDocument/semanticTokens/full",
            params: lastResultId
              ? {
                  textDocument: { uri: URI },
                  previousResultId: lastResultId,
                }
              : {
                  textDocument: { uri: URI },
                },
          });

          if (!output.response) {
            return {
              data: new Uint32Array(),
            };
          }

          const parsed = JSON.parse(output.response) as {
            result?: {
              resultId?: string;
              data?: number[];
              edits?: Array<{ start: number; deleteCount: number; data?: number[] }>;
            };
          };

          if (parsed.result?.edits && parsed.result.resultId) {
            return {
              resultId: parsed.result.resultId,
              edits: parsed.result.edits.map((edit) => ({
                start: edit.start,
                deleteCount: edit.deleteCount,
                data: new Uint32Array(edit.data ?? []),
              })),
            };
          }

          return {
            resultId: parsed.result?.resultId ?? "",
            data: new Uint32Array(parsed.result?.data ?? []),
          };
        },
        releaseDocumentSemanticTokens() {},
      });

      sourceEditor = monaco.editor.create(sourceEditorRef!, {
        model: sourceModel,
        minimap: { enabled: false },
        fontSize: 14,
        automaticLayout: true,
        theme: "vs-dark",
      });

      tsViewEditor = monaco.editor.create(tsViewRef!, {
        model: tsModel,
        minimap: { enabled: false },
        fontSize: 13,
        automaticLayout: true,
        theme: "vs-dark",
        readOnly: true,
      });

      const tsMarkersDisposable = monaco.editor.onDidChangeMarkers((resources) => {
        if (resources.some((resource) => resource.toString() === tsModel.uri.toString())) {
          syncTsProblems();
        }
      });

      createEffect(() => {
        if (activeRightTab() === "ts") {
          requestAnimationFrame(() => tsViewEditor?.layout());
        }
      });

      syncDocument(currentText);

      const changeDisposable = sourceEditor.onDidChangeModelContent(() => {
        syncDocument(sourceModel.getValue());
      });

      setReady(true);

      onCleanup(() => {
        invokeLsp({
          jsonrpc: "2.0",
          method: "textDocument/didClose",
          params: { textDocument: { uri: URI } },
        });
        changeDisposable.dispose();
        tsMarkersDisposable.dispose();
        sourceEditor?.dispose();
        tsViewEditor?.dispose();
        sourceModel.dispose();
        tsModel.dispose();
      });
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    }
  });

  return (
    <main class="app-root">
      <header class="header">
        <h1>Lumo Web Playground</h1>
        <p>Write Lumo (left), inspect emitted TypeScript (right), run JS.</p>
      </header>

      <section class="workspace">
        <section class="source-pane">
          <div class="editor" ref={sourceEditorRef} />
          <section class="problems-panel">
            <h2>Problems</h2>
            <Show when={problems().length > 0} fallback={<p class="hint">No problems.</p>}>
              <ul class="problems-list">
                <For each={problems()}>
                  {(problem) => (
                    <li class="problem-item">
                      {`L${problem.start_line + 1}:${problem.start_character + 1}-L${
                        problem.end_line + 1
                      }:${problem.end_character + 1} Problem ${problem.message}`}
                    </li>
                  )}
                </For>
              </ul>
            </Show>
          </section>
        </section>

        <aside class="sidebar">
          <div class="panel-row panel-controls">
            <button
              class="run-btn"
              type="button"
              onClick={async () => setRuntimeOutput(await runGeneratedJs(emittedJs()))}
              disabled={!emittedJs()}
            >
              Run JS
            </button>
          </div>

          <div class="tabs">
            <button
              class={`tab-btn ${activeRightTab() === "ts" ? "active" : ""}`}
              type="button"
              onClick={() => setActiveRightTab("ts")}
            >
              TypeScript
            </button>
            <button
              class={`tab-btn ${activeRightTab() === "typed-ast" ? "active" : ""}`}
              type="button"
              onClick={() => setActiveRightTab("typed-ast")}
            >
              Lumo Typed AST
            </button>
            <button
              class={`tab-btn ${activeRightTab() === "lowered-ast" ? "active" : ""}`}
              type="button"
              onClick={() => setActiveRightTab("lowered-ast")}
            >
              Lumo IR Lowered Typed AST
            </button>
          </div>

          <section class="ts-panel" classList={{ "is-hidden": activeRightTab() !== "ts" }}>
            <div class="ts-view" ref={tsViewRef} />
            <section class="ts-problems-panel">
              <div class="subpanel-header">
                <h2>TS Problems</h2>
                <span class="problems-badge">{tsProblems().length}</span>
              </div>
              <Show when={tsProblems().length > 0} fallback={<p class="hint">No TS problems.</p>}>
                <ul class="problems-list ts-problems-list">
                  <For each={tsProblems()}>
                    {(problem) => {
                      const severity = formatMarkerSeverity(problem.severity);
                      return (
                        <li class={`problem-item ts-problem-item severity-${severity}`}>
                          <span class="problem-severity">{severity}</span>
                          <span>
                            {`L${problem.startLineNumber}:${problem.startColumn}-L${problem.endLineNumber}:${problem.endColumn} ${problem.message}`}
                          </span>
                        </li>
                      );
                    }}
                  </For>
                </ul>
              </Show>
            </section>
          </section>
          <pre class="ast-view" classList={{ "is-hidden": activeRightTab() !== "typed-ast" }}>
            {typedAst() || "(no typed AST)"}
          </pre>
          <pre class="ast-view" classList={{ "is-hidden": activeRightTab() !== "lowered-ast" }}>
            {loweredTypedAst() || "(no lowered typed AST)"}
          </pre>

          <Show when={runtimeOutput().length > 0}>
            <h2>Output</h2>
            <pre class="runtime-output">{runtimeOutput().join("\n")}</pre>
          </Show>

          <Show when={!ready()}>
            <p>Loading wasm + editor…</p>
          </Show>
          <Show when={error()}>{(message) => <p class="error">{message()}</p>}</Show>
          <Show when={!emittedTs() && ready()}>
            <p class="hint">No TS output yet.</p>
          </Show>
        </aside>
      </section>
    </main>
  );
}
