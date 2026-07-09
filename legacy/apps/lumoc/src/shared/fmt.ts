const Parens = {
  open: '({⟨',
  close: ')}⟩',
};
const maxInlineLength = 25;

export function formatParens(source: string): string {
  let result = '';
  let indentDepth = 0;
  for (let i = 0; i < source.length; i++) {
    const c = source[i]!;
    if (Parens.open.includes(c)) {
      if (
        Array.from({ length: maxInlineLength }).some((_, j) => {
          const between = source.slice(i + 1, i + j);
          const ch = source.at(i + j);
          return (
            ch != null &&
            Parens.close.includes(ch) &&
            Array.from(between)
              .map(
                (ch) =>
                  (Parens.open.includes(ch)
                    ? 1
                    : Parens.close.includes(ch)
                    ? -1
                    : ch === ','
                    ? NaN
                    : 0) as number,
              )
              .reduce((acc, curr) => acc + curr, 0) === 0
          );
        })
      ) {
        result += c;
      } else {
        indentDepth++;
        result += c;
        result += '\n';
        result += '  '.repeat(indentDepth);
      }
    } else if (Parens.close.includes(c)) {
      if (
        Array.from({ length: maxInlineLength }).some((_, j) => {
          const between = source.slice(i - j + 1, i);
          const ch = source.at(i - j);
          return (
            ch != null &&
            Parens.open.includes(ch) &&
            Array.from(between)
              .map(
                (ch) =>
                  (Parens.open.includes(ch)
                    ? 1
                    : Parens.close.includes(ch)
                    ? -1
                    : ch === ','
                    ? NaN
                    : 0) as number,
              )
              .reduce((acc, curr) => acc + curr, 0) === 0
          );
        })
      ) {
        result += c;
      } else {
        result += '\n';
        indentDepth--;
        result += '  '.repeat(indentDepth);
        result += c;
      }
    } else if (c === '\n') {
      result += c;
      result += '  '.repeat(indentDepth);
    } else if (c === ',') {
      result += c;
      result += '\n';
      result += '  '.repeat(indentDepth);
    } else if (c === ' ' && source.at(i - 1) === ',') {
    } else {
      result += c;
    }
  }
  return result;
}
