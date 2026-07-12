// The D-43 prelude: the free identifiers that emitted Lumo JS expects.
// `handle` pushes a bundle onto a dynamically-scoped per-cap stack;
// `perform` reads the innermost one.
const __lumo_handlers = {};
function __lumo_handle(cap, bundle, body) {
  (__lumo_handlers[cap] ||= []).push(bundle);
  try {
    return body();
  } finally {
    __lumo_handlers[cap].pop();
  }
}
function __lumo_perform(cap) {
  const stack = __lumo_handlers[cap];
  if (!stack || stack.length === 0) {
    throw new Error(`unhandled capability: ${cap}`);
  }
  return stack[stack.length - 1];
}
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
// `use` lowers to `require()(...)` (D-30); no module system in the browser.
const require = () => (name) => {
  throw new Error(`require(${JSON.stringify(name)}) is not available in the playground`);
};
