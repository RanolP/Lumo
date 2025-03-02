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

  !b(q,z) + a * -x + y + !e * z + d
  -f(x,-z(q))
  {a;b;;;;c
  d
  e
  f;g
  h}
  Nat.S(.S(.S(.O)))
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

  const restTokens = rest.intoInner;
  if (restTokens.length > 0) {
    console.log('Rest tokens:\n======\n' + restTokens.join('\n') + '\n=======');
  }

  ast.forEach((item) => {
    console.log(formatAst(item.toString()));
  });
} catch (e) {
  if (e instanceof ParseError && e.input instanceof ArrayInput) {
    console.error(e.message);
    console.log(e.input.intoInner);
  } else {
    throw e;
  }
}
