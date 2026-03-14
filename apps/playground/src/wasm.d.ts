declare module "./wasm/lumo_playground_wasm.js" {
  export default function init(moduleOrPath?: string | URL | Response | BufferSource | WebAssembly.Module): Promise<void>;

  export class WasmCompiler {
    constructor();
    set_file(uri: string, source: string): void;
    compile(uri: string): {
      parse_errors: Diagnostic[];
      diagnostics: Diagnostic[];
      stats: {
        parse_computes: number;
        lower_computes: number;
        diagnostics_computes: number;
      };
      typed_ast: string;
      lowered_typed_ast: string;
      lowered: string;
      emitted_ts: string;
      emitted_d_ts: string;
      emitted_js: string;
      backend_errors: string[];
    };
  }

  export class WasmLsp {
    constructor();
    handle_message(message: string): {
      response?: string;
      notifications: string[];
      should_exit: boolean;
    };
  }

  interface Diagnostic {
    start: number;
    end: number;
    start_line: number;
    start_character: number;
    end_line: number;
    end_character: number;
    message: string;
  }
}
