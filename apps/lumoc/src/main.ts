import { TsLoweringContext } from './features/codegen/ts/lowering';
import { IrLexer } from './features/syntax/lexer';
import { program } from './features/syntax/parser';
import type { ParserInput } from './features/syntax/parser/base';
import { TypeV } from './features/type';
import { Typer } from './features/typer';
import { emitExpr } from './lib/simple-ts-ast/emit';
import { formatParens } from './shared/fmt';
import { ParseError } from './vendors/malssi/parser/errors';
import {
  createArrayInput,
  createContextfulInput,
} from './vendors/malssi/parser/input';
import fs from 'node:fs/promises';
import process from 'node:process';

const source = await fs.readFile('./main.lumo', 'utf-8');
const tokens = IrLexer.lex(source);

const input = createContextfulInput({ isBlock: false, id: 0 })(
  createArrayInput(tokens),
);
let result;
try {
  result = program.run(input);
} catch (e) {
  if (e instanceof ParseError) {
    console.error(
      (e.input as ParserInput).leftoverTokens.map((t) => t.display()).join(''),
    );
  }
  throw e;
}
if (
  input.leftoverTokens.length > 0 &&
  input.leftoverTokens.some((t) => !t.VerticalSpace && !t.HorizontalSpace)
) {
  console.error(
    `parse error on ${input.leftoverTokens
      .map((token) => token.display())
      .join('')}`,
  );
  process.exit(1);
}

for (const [name, type] of result.typedefs) {
  console.log(`type ${name} = ${type.display()}`);
}

console.log(formatParens(result.main.display()));

const typer = Typer.create();

for (const [name, type] of result.typedefs) {
  typer.unify_v(type, TypeV.Variable(name).freshRefined());
}

const typedMain = typer.infer_c(result.main);

console.log('======================');
console.log(formatParens(typedMain.display()));
console.log('======================');

const loweringContext = new TsLoweringContext();
for (const [name, type] of result.typedefs) {
  loweringContext.lower_t_v(type, [], {
    text: name,
    onRaw: () => {},
    onNumbered: () => {
      throw new Error('never fails');
    },
  });
}
const tsMain = loweringContext.lower_c(typedMain, []);
const tsSource = `
const STD_TAG = Symbol('Lumo/tag');

const std = {
  Never: (): never => { throw new Error('Never') },
};

${loweringContext.emitTsTypes()}

const LumoModule = ${formatParens(emitExpr(tsMain))}
`;

await fs.mkdir('./dist', { recursive: true });
await fs.writeFile('./dist/out.ts', tsSource);
