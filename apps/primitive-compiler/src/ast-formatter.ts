import { match, P } from 'ts-pattern';

export function formatAst(s: string) {
  let result = '';

  let prev: string | null = null;

  let indent = 0;
  for (const i of Array.from({ length: s.length }).map((_, idx) => idx)) {
    result += s[i];
    const slice = s.slice(Math.max(0, i - 1), Math.min(i + 2, s.length));
    match([slice[0], slice[1], slice[2]])
      .with([P.union('(', '['), '\n', P.any], () => {
        indent += 2;
      })
      .with(P.union([P.any, '\n', P.union(')', ']')]), () => {
        indent -= 2;
      })
      .otherwise(() => {});
    if (s[i] === '\n') {
      result += ' '.repeat(Math.max(indent, 0));
    }
  }

  return result;
}
