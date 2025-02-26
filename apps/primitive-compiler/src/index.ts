import '@total-typescript/ts-reset';

import dedent from 'dedent';
import { Tokenizer } from './#syntax';
import { ArrayInput } from './#lib/parsecom';
import { ParseError } from './#lib/parsecom/error';
import { formatAst } from './ast-formatter';
import { program } from './#syntax/parser';

const source = dedent`
  enum Nat {
    O,
    S(Nat),
  }
`;

const tokens = Array.from(new Tokenizer(source));
// console.log(tokens.join('\n'));
try {
  const [rest, ast] = program(
    new ArrayInput(tokens, {
      nodeId: 0,
      minimumBindingPower: 0,
      newlineAsSemi: false,
    }),
  );

  ast.forEach((item) => {
    console.log(formatAst(item.toString()));
  });
} catch (e) {
  if (e instanceof ParseError && e.input instanceof ArrayInput) {
    console.error(e.message);
    console.log(e.input.intoInner);
  }
}
