import { malssi } from '../../../vendors/malssi/parser';
import {
  type ContextfulInput,
  type ArrayInput,
} from '../../../vendors/malssi/parser/input';
import { Token } from '../lexer';

export type ParserInput = ContextfulInput<
  { isBlock: boolean; id: number },
  ArrayInput<Token>
>;

export const parser = malssi<ParserInput>();

export const ctx = {
  freshId: parser.raw((i) => i.context.id++),
};
