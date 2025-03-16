import '@total-typescript/ts-reset';

import { Token, Tokenizer, TokenKind } from './#syntax/index.js';
import { ArrayInput } from './#lib/parsecom/index.js';
import { ParseError } from './#lib/parsecom/error.js';
import { formatAst } from './ast-formatter.js';
import { program } from './#syntax/parser/index.js';
import fs from 'node:fs/promises';
import { TypeScope } from './#core/#type/index.js';
import { EnumDefinition, FunctionDefinition } from './#core/#ast/definition.js';
import { infer, visit } from './#type/index.js';
import { TypingError } from './#core/#type/error.js';
import { TypeVar } from './#core/#type/variants.js';
import { match } from 'ts-pattern';

const source = await fs.readFile('source.lumo', { encoding: 'utf-8' });

interface Line {
  position: number;
  tokens: Token[];
}

const tokens = Array.from(new Tokenizer(source));
const lines: Line[] = [{ position: 0, tokens: [] }];
for (const token of tokens) {
  const lastLine = lines[lines.length - 1];
  if (token.kind === TokenKind.SpaceVertical) {
    lines.push({
      position: lastLine.position + lastLine.tokens.length,
      tokens: [],
    });
  } else {
    lastLine.tokens.push(token);
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

  // ast.forEach((node) => {
  //   console.log(formatAst(node.toString()));
  // });

  const scope = new TypeScope();
  const expressions = [];
  for (const node of ast) {
    if (node instanceof EnumDefinition || node instanceof FunctionDefinition) {
      visit(scope, node);
    } else {
      expressions.push(node);
    }
  }
  for (const expr of expressions) {
    try {
      const ty = infer(scope, expr);
      const exprStr = formatAst(expr.toString());
      console.log(`${exprStr} :: ${ty}`);
      if (ty instanceof TypeVar) {
        console.log(
          `${' '.repeat(exprStr.split('\n').at(-1)!.length)} == ${scope.lookup(
            ty.path,
          )}`,
        );
      }
    } catch (e) {
      handleError(e);
    }
  }
} catch (e) {
  handleError(e);
}

function handleError(e: unknown) {
  if (e instanceof ParseError && e.input instanceof ArrayInput) {
    reportDiagnostic(
      e.message,
      {
        kind: 'position',
        position: e.input.position,
      },
      (e.input as ArrayInput<Token, unknown>).intoInner.at(e.input.position)
        ?.content.length ?? 1,
    );
  } else if (e instanceof TypingError) {
    if (e.node == null) {
      console.error(e.message);
    } else {
      reportDiagnostic(
        formatAst(e.message),
        {
          kind: 'bytes-index',
          byteIndex: e.node.span.begin + 1n,
        },
        Number(e.node.span.end - e.node.span.begin),
      );
    }
  } else {
    throw e;
  }
}

function reportDiagnostic(
  message: string,
  lookup:
    | { kind: 'position'; position: number }
    | { kind: 'bytes-index'; byteIndex: bigint },
  width: number,
) {
  const lineNo =
    lines.length -
    [...lines].reverse().findIndex((l) =>
      match(lookup)
        .with({ kind: 'position' }, ({ position }) => l.position < position)
        .with(
          { kind: 'bytes-index' },
          ({ byteIndex }) => l.tokens[0].span.begin < byteIndex,
        )
        .exhaustive(),
    );
  const line = lines[lineNo - 1];
  const colNo = match(lookup)
    .with({ kind: 'position' }, ({ position }) =>
      line.tokens
        .slice(0, position - line.position)
        .reduce((acc, token) => acc + token.content.length, 1),
    )
    .with(
      { kind: 'bytes-index' },
      ({ byteIndex }) => byteIndex - line.tokens[0].span.begin,
    )
    .exhaustive();

  const prefix = `source.lumo:${lineNo}:${colNo} `;
  console.error(
    `${prefix}${message
      .split('\n')
      .map((v, i) => (i === 0 ? v : `${' '.repeat(prefix.length)}${v}`))
      .join('\n')}`,
  );
  const indentLength = lineNo.toString().length + 1;
  console.error(`${''.padStart(indentLength)} | `);
  console.error(
    `${lineNo.toString().padStart(indentLength)} | ` +
      line.tokens.map((t) => t.content).join(''),
  );
  console.error(`${''.padStart(indentLength)} |${' '.repeat(Number(colNo))}^`);
  console.error(`${''.padStart(indentLength)} | `);
}
