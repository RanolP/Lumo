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
// `use` lowers to `require()(...)` (D-30); no module system in the browser.
const require = () => (name) => {
  throw new Error(`require(${JSON.stringify(name)}) is not available in the playground`);
};
