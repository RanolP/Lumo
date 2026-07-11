import { createSignal, For, onCleanup, onMount, Show } from "solid-js";
import * as monaco from "monaco-editor/esm/vs/editor/editor.api";
import "monaco-editor/esm/vs/basic-languages/javascript/javascript.contribution";
import editorWorker from "monaco-editor/esm/vs/editor/editor.worker?worker";
import runtimePrelude from "./lumo-runtime.js?raw";

declare global {
  interface Window {
    MonacoEnvironment?: {
      getWorker: (_moduleId: string, _label: string) => Worker;
    };
  }
}

window.MonacoEnvironment = {
  getWorker: () => new editorWorker(),
};

const LANGUAGE_ID = "lumo";
const INITIAL_SOURCE = `// Peano numbers, a match, and a handled capability — all four
// views (JS / MIR / Types / Parse tree) light up. Hit Run to
// execute the emitted JS.

data Nat {
  .zero
  .succ(Nat)
}

fn pred(n: Nat): Nat / {} {
  match n {
    .zero => Nat.zero,
    .succ(m) => m,
  }
}

cap Console {
  fn log(msg: Nat): Nat
}

fn main(): Nat / {} {
  handle Console with bundle { fn log(m: Nat) { m } } in
  Console.log(pred(Nat.succ(Nat.succ(Nat.zero))))
}
`;

// Keyword list mirrors lumo/Lumo.tokens.syn.langue.
const KEYWORDS = [
  "fn", "let", "in", "data", "cap", "extern", "use", "impl", "match",
  "if", "else", "handle", "with", "for", "thunk", "force", "bundle",
  "perform", "type",
];

// Mirrors `ParseDiag` / `CompileResult` in crates/playground-wasm.
type ParseDiag = {
  start_line: number;
  start_character: number;
  end_line: number;
  end_character: number;
  message: string;
};
type CompileResult = {
  js: string;
  js_errors: string[];
  mir: string;
  mir_errors: string[];
  types: string;
  type_errors: string[];
  sexpr: string;
  parse_diags: ParseDiag[];
};

type Problem = { location: string; message: string };
type RightTab = "js" | "mir" | "types" | "tree";

function collectProblems(result: CompileResult): Problem[] {
  const problems: Problem[] = result.parse_diags.map((diag) => ({
    location: `L${diag.start_line + 1}:${diag.start_character + 1}-L${diag.end_line + 1}:${
      diag.end_character + 1
    }`,
    message: diag.message,
  }));
  // With parse errors present, downstream stages just restate them.
  if (problems.length > 0) return problems;
  const seen = new Set<string>();
  for (const [source, messages] of [
    ["JS", result.js_errors],
    ["MIR", result.mir_errors],
    ["Types", result.type_errors],
  ] as const) {
    for (const message of messages) {
      if (seen.has(message)) continue;
      seen.add(message);
      problems.push({ location: source, message });
    }
  }
  return problems;
}

function runGeneratedJs(code: string): Promise<string[]> {
  return new Promise((resolve) => {
    const workerSource = [
      "const __logs = [];",
      "const __formatValue = (value) => {",
      "  if (typeof value === 'string') return value;",
      "  try { return JSON.stringify(value); } catch { return String(value); }",
      "};",
      "const __formatError = (cause) => {",
      "  if (cause instanceof Error) return cause.message;",
      "  try { return JSON.stringify(cause); } catch { return String(cause); }",
      "};",
      "globalThis.console = {",
      "  log: (...args) => { __logs.push(args.map(__formatValue).join(' ')); },",
      "  info: (...args) => { __logs.push(`[info] ${args.map(__formatValue).join(' ')}`); },",
      "  warn: (...args) => { __logs.push(`[warn] ${args.map(__formatValue).join(' ')}`); },",
      "  error: (...args) => { __logs.push(`[error] ${args.map(__formatValue).join(' ')}`); },",
      "};",
      "try {",
      runtimePrelude,
      code,
      "  ;(() => {",
      "    if (typeof main === 'function') {",
      "      const __result = main();",
      "      if (__result !== undefined) __logs.push(`[result] ${__formatValue(__result)}`);",
      "    } else {",
      "      __logs.push('[info] no `fn main()` to run');",
      "    }",
      "  })();",
      "} catch (cause) {",
      "  __logs.push(`[runtime error] ${__formatError(cause)}`);",
      "}",
      "if (__logs.length === 0) __logs.push('(program executed with no output)');",
      "postMessage({ logs: __logs });",
    ].join("\n");

    const blob = new Blob([workerSource], { type: "text/javascript" });
    const url = URL.createObjectURL(blob);
    const worker = new Worker(url);
    let settled = false;

    const finish = (logs: string[]) => {
      if (settled) return;
      settled = true;
      worker.terminate();
      URL.revokeObjectURL(url);
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
      finish([`[runtime error] ${event.message || "worker execution failed"}`]);
    };
  });
}

export default function App() {
  let sourceEditorRef: HTMLDivElement | undefined;
  let jsViewRef: HTMLDivElement | undefined;
  let sourceEditor: monaco.editor.IStandaloneCodeEditor | undefined;
  let jsViewEditor: monaco.editor.IStandaloneCodeEditor | undefined;

  const [ready, setReady] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const [emittedJs, setEmittedJs] = createSignal("");
  const [mirView, setMirView] = createSignal("");
  const [typesView, setTypesView] = createSignal("");
  const [treeView, setTreeView] = createSignal("");
  const [activeRightTab, setActiveRightTab] = createSignal<RightTab>("js");
  const [runtimeOutput, setRuntimeOutput] = createSignal<string[]>([]);
  const [problems, setProblems] = createSignal<Problem[]>([]);

  onMount(async () => {
    try {
      const wasm = await import("./wasm/lumo_playground_wasm.js");
      await wasm.default();

      monaco.languages.register({ id: LANGUAGE_ID });
      monaco.languages.setMonarchTokensProvider(LANGUAGE_ID, {
        keywords: KEYWORDS,
        tokenizer: {
          root: [
            [/\/\/.*/, "comment"],
            [/"([^"\\]|\\.)*"/, "string"],
            [/[0-9]+(\.[0-9]+)?/, "number"],
            [
              /[a-zA-Z_][a-zA-Z0-9_]*/,
              { cases: { "@keywords": "keyword", "@default": "identifier" } },
            ],
            [/=>|->|==|!=|<=|>=|&&|\|\||\*\*|\.\./, "operator"],
            [/[{}()[\]]/, "@brackets"],
            [/[=+\-*/%!<>:;,.#_]/, "operator"],
            [/\s+/, "white"],
          ],
        },
      });

      const sourceModel = monaco.editor.createModel(
        INITIAL_SOURCE,
        LANGUAGE_ID,
        monaco.Uri.parse("file:///main.lumo"),
      );
      const jsModel = monaco.editor.createModel(
        "",
        "javascript",
        monaco.Uri.parse("file:///out.js"),
      );

      const compileNow = () => {
        const source = sourceModel.getValue();
        const result = wasm.compile(source) as CompileResult;

        setEmittedJs(result.js);
        setMirView(result.mir);
        setTypesView(result.types);
        setTreeView(result.sexpr);
        setProblems(collectProblems(result));
        jsModel.setValue(result.js);

        monaco.editor.setModelMarkers(
          sourceModel,
          "lumo-compiler",
          result.parse_diags.map((diag) => ({
            message: diag.message,
            severity: monaco.MarkerSeverity.Error,
            startLineNumber: diag.start_line + 1,
            startColumn: diag.start_character + 1,
            endLineNumber: diag.end_line + 1,
            endColumn: diag.end_character + 1,
          })),
        );
      };

      sourceEditor = monaco.editor.create(sourceEditorRef!, {
        model: sourceModel,
        minimap: { enabled: false },
        fontSize: 14,
        automaticLayout: true,
        theme: "vs-dark",
      });

      jsViewEditor = monaco.editor.create(jsViewRef!, {
        model: jsModel,
        minimap: { enabled: false },
        fontSize: 13,
        automaticLayout: true,
        theme: "vs-dark",
        readOnly: true,
        wordWrap: "on",
      });

      compileNow();

      let debounce: ReturnType<typeof setTimeout> | undefined;
      const changeDisposable = sourceEditor.onDidChangeModelContent(() => {
        clearTimeout(debounce);
        debounce = setTimeout(compileNow, 150);
      });

      setReady(true);

      onCleanup(() => {
        clearTimeout(debounce);
        changeDisposable.dispose();
        sourceEditor?.dispose();
        jsViewEditor?.dispose();
        sourceModel.dispose();
        jsModel.dispose();
      });
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    }
  });

  const tabs: Array<{ id: RightTab; label: string }> = [
    { id: "js", label: "JS" },
    { id: "mir", label: "MIR" },
    { id: "types", label: "Types" },
    { id: "tree", label: "Parse tree" },
  ];

  return (
    <main class="app-root">
      <header class="header">
        <h1>Lumo Web Playground</h1>
        <p>Write Lumo (left), inspect the pipeline output (right), run the emitted JS.</p>
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
                    <li class="problem-item">{`${problem.location} ${problem.message}`}</li>
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
            <For each={tabs}>
              {(tab) => (
                <button
                  class={`tab-btn ${activeRightTab() === tab.id ? "active" : ""}`}
                  type="button"
                  onClick={() => setActiveRightTab(tab.id)}
                >
                  {tab.label}
                </button>
              )}
            </For>
          </div>

          <div
            class="ts-view js-view"
            classList={{ "is-hidden": activeRightTab() !== "js" }}
            ref={jsViewRef}
          />
          <pre class="ast-view" classList={{ "is-hidden": activeRightTab() !== "mir" }}>
            {mirView() || "(no MIR output)"}
          </pre>
          <pre class="ast-view" classList={{ "is-hidden": activeRightTab() !== "types" }}>
            {typesView() || "(no inferred types)"}
          </pre>
          <pre class="ast-view" classList={{ "is-hidden": activeRightTab() !== "tree" }}>
            {treeView() || "(no parse tree)"}
          </pre>

          <Show when={runtimeOutput().length > 0}>
            <h2>Output</h2>
            <pre class="runtime-output">{runtimeOutput().join("\n")}</pre>
          </Show>

          <Show when={!ready() && !error()}>
            <p>Loading wasm + editor…</p>
          </Show>
          <Show when={error()}>{(message) => <p class="error">{message()}</p>}</Show>
        </aside>
      </section>
    </main>
  );
}
