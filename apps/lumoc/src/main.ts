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

const source = await fs.readFile('./main.lumo', 'utf-8');
const tokens = IrLexer.lex(source);

const result = program.run(
  createContextfulInput({ isBlock: false, id: 0 })(createArrayInput(tokens)),
);

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
