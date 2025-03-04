import '@total-typescript/ts-reset';

import { Token, Tokenizer, TokenKind } from './#syntax/index.js';
import { ArrayInput } from './#lib/parsecom/index.js';
import { ParseError } from './#lib/parsecom/error.js';
import { formatAst } from './ast-formatter.js';
import { program } from './#syntax/parser/index.js';
import fs from 'node:fs/promises';

const source = await fs.readFile('source.lumo', { encoding: 'utf-8' });

interface Line {
  position: number;
  tokens: Token[];
}

const tokens = Array.from(new Tokenizer(source));
const lines: Line[] = [{ position: 0, tokens: [] }];
for (const token of tokens) {
  const lastLine = lines[lines.length - 1];
  lastLine.tokens.push(token);
  if (token.kind === TokenKind.SpaceVertical) {
    lines.push({
      position: lastLine.position + lastLine.tokens.length,
      tokens: [],
    });
  }
}
// console.log(tokens.join('\n'));
try {
  const [rest, ast] = program(
    new ArrayInput(tokens, {
      nodeId: 0,
      minimumBindingPower: 0,
      newlineAsSemi: false,
    }),
  );

  const restTokens = rest.intoInner;
  if (restTokens.length > 0) {
    console.log('Rest tokens:\n======\n' + restTokens.join('\n') + '\n=======');
  }

  ast.forEach((item) => {
    console.log(formatAst(item.toString()));
  });
} catch (e) {
  if (e instanceof ParseError && e.input instanceof ArrayInput) {
    const lineNo =
      lines.length -
      [...lines].reverse().findIndex((l) => l.position < e.input.position);
    const line = lines[lineNo - 1];
    const colNo = line.tokens
      .slice(0, e.input.position - line.position)
      .reduce((acc, token) => acc + token.content.length, 1);
    console.log(`source.lumo:${lineNo}:${colNo} ${e.message}`);
    const indentLength = lineNo.toString().length + 1;
    console.log(`${''.padStart(indentLength)} | `);
    console.log(
      `${lineNo.toString().padStart(indentLength)} | ` +
        line.tokens
          .map((t) => t.content)
          .join('')
          .trim(),
    );
    console.log(`${''.padStart(indentLength)} | `);
  } else {
    throw e;
  }
}
