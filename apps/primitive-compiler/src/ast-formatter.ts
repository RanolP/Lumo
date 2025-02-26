export function formatAst(s: string) {
  let result = '';

  let isSpan = false;

  let indent = 0;
  for (const c of s) {
    switch (c) {
      case '(':
      case '[':
        if (result.endsWith('Span')) {
          isSpan = true;
        }
        if (!isSpan) {
          indent += 2;
        }
      case ',':
        result += c;
        if (!isSpan) {
          result += '\n';
          result += ' '.repeat(indent);
        }
        break;
      case ')':
      case ']':
        if (!isSpan) {
          indent -= 2;
          result += '\n';
          result += ' '.repeat(indent);
        }
        isSpan = false;
        result += c;
        break;
      case ' ':
        break;
      default:
        result += c;
        break;
    }
  }

  return result;
}
