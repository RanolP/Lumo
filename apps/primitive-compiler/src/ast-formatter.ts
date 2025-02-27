import { match, P } from 'ts-pattern';

export function formatAst(s: string) {
  let result = '';

  let prev: string | null = null;

  let indent = 0;
  for (const c of s) {
    result += c;
    match([prev, c] as const)
      .with([P.union('(', '['), '\n'], () => {
        indent += 2;
      })
      .with([P.union(')', ']'), '\n'], () => {
        indent -= 2;
      })
      .otherwise(() => {});
    if (c === '\n') {
      result += ' '.repeat(Math.max(indent, 0));
    }
    prev = c;
  }

  return result;
}
