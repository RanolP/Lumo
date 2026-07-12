// The node runtime prelude for the ported stdlib (D-45): the D-43 core
// (__lumo_*) plus the host bindings for every `extern fn` in
// packages/*. Each binding is a thunked n-ary function matching the
// `force f(args)` calling convention (`f()(a, b)`). This file is the
// source of truth for what an extern does in JS — the backend has no
// extern-mapping attributes (D-43).

// === D-43 core ===
// No handler machinery: capabilities pass lexically (D-51) —
// `__lumo_perform`/`__lumo_handle` exist only for MIR-level programs
// that spell `perform` directly, which the stdlib never does.
function __lumo_match_error(value) {
  throw new Error(`match error: no arm matches ${JSON.stringify(value)}`);
}

// D-54 abortive-handler boundary: `try`/`abort` in MIR. The token is
// a fresh object per boundary entry, so nested handles of the same
// cap stay precise; foreign exceptions pass through.
const __lumo_boundary = (f) => {
  const tok = {};
  try {
    return f(tok);
  } catch (e) {
    if (e && e.__lumo_tok === tok) return e.value;
    throw e;
  }
};
const __lumo_abort = (tok, value) => {
  throw { __lumo_tok: tok, value };
};

// Lumo Bool is the tagged `data Bool { .true, .false }` encoding.
const __lumo_true = { $: "true", args: [] };
const __lumo_false = { $: "false", args: [] };
const __lumo_bool = (b) => (b ? __lumo_true : __lumo_false);

// === libcore/src#js/number.lumo ===
const __num_add = () => (a, b) => a + b;
const __num_sub = () => (a, b) => a - b;
const __num_mul = () => (a, b) => a * b;
const __num_div = () => (a, b) => a / b;
const __num_mod = () => (a, b) => a % b;
const __num_neg = () => (a) => -a;
const __num_floor = () => (a) => Math.floor(a);
const __num_eq = () => (a, b) => __lumo_bool(a === b);
const __num_lt = () => (a, b) => __lumo_bool(a < b);

// === libcore/src#js/string.lumo ===
const __str_len = () => (s) => s.length;
const __str_char_at = () => (s, idx) => s.charAt(idx);
const __str_slice = () => (s, start, end) => s.slice(start, end);
const __str_concat = () => (a, b) => a + b;
const __str_eq = () => (a, b) => __lumo_bool(a === b);
const __str_starts_with = () => (s, prefix) => __lumo_bool(s.startsWith(prefix));
const __str_contains = () => (s, sub) => __lumo_bool(s.includes(sub));
const __str_index_of = () => (s, sub) => s.indexOf(sub);
const __str_trim = () => (s) => s.trim();
const __char_code_at = () => (s, idx) => s.charCodeAt(idx);
const __from_char_code = () => (code) => String.fromCharCode(code);
const __str_replace_all = () => (s, from, to) => s.replaceAll(from, to);
const __num_to_string = () => (n) => n.toString();

// === libstd/src#js/io.lumo ===
const __println = () => (msg) => console.log(msg);

// === libstd/src#js.node/fs.lumo ===
const readFileSync = () => (path, encoding) =>
  require("node:fs").readFileSync(path, encoding);
const writeFileSync = () => (path, content, encoding) =>
  require("node:fs").writeFileSync(path, content, encoding);

// === libstd/src#js.node/process.lumo ===
const __argv_at_raw = () => (idx) => process.argv[idx];
// Nullary: a Lumo call `f()` is one JS call (`force` collapses with
// the empty application), so the binding is single-thunked.
const __argv_length_raw = () => process.argv.length;
const __exit_process = () => (code) => process.exit(code);
const __console_error = () => (msg) => console.error(msg);
