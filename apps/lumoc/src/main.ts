import { IrLexer } from './features/ir-syntax/lexer';
import { program } from './features/ir-syntax/parser';
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

console.log(result.main.display());
