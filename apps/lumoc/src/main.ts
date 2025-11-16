import { IrLexer } from './features/ir-syntax/lexer';
import { program } from './features/ir-syntax/parser';
import { TypeV } from './features/type';
import { Typer } from './features/typer';
import { formatParens } from './shared/fmt';
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
const result = program.run(input);
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

for (const { name, type } of result.typedefs) {
  console.log(`type ${name} = ${type.display()}`);
}

console.log(formatParens(result.main.display()));

const typer = Typer.create();

for (const { name, type } of result.typedefs) {
  typer.unify_v(type, TypeV.Variable(name).freshRefined());
}

const typedMain = typer.infer_c(result.main);

console.log(formatParens(typedMain.display()));
