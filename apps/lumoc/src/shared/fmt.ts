export function formatParens(source: string): string {
  let result = '';
  let indentDepth = 0;
  for (let i = 0; i < source.length; i++) {
    const c = source[i]!;
    if (
      '({⟨'.includes(c) &&
      !')}⟩'.includes(source.at(i + 1)!) &&
      !')}⟩'.includes(source.at(i + 2)!)
    ) {
      indentDepth++;
      result += c;
      result += '\n';
      result += '  '.repeat(indentDepth);
    } else if (')}⟩'.includes(c)) {
      if ('({⟨'.includes(source.at(i - 1)!)) {
        result += ' ';
        result += c;
      } else if ('({⟨'.includes(source.at(i - 2)!)) {
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
    } else {
      result += c;
    }
  }
  return result;
}
