import { PathType, TupleType, Type } from '@/#core/#ast/index.js';
import { TokenKind } from '../../common/index.js';
import {
  cut,
  oneof,
  opt,
  Parser,
  rule,
  separatedList1,
  seq,
  token,
} from '../base.js';
import { astId, path } from './fragments.js';

export const type: Parser<Type> = rule(() =>
  oneof('type', types.tuple, types.path),
);

const types = {
  tuple: rule(() =>
    seq(
      token(TokenKind.PunctuationLeftParenthesis),
      cut(
        seq(
          opt(
            seq(
              type,
              token(TokenKind.PunctuationComma),
              opt(separatedList1(type, token(TokenKind.PunctuationComma))),
              opt(token(TokenKind.PunctuationComma)),
            ),
          ),
          token(TokenKind.PunctuationRightParenthesis),
        ),
      ),
      astId,
    ),
  ).map(([_0, [body, _1], id]) => {
    let types: Type[] = [];
    if (body != null) {
      const [ty, _2, repeat, _3] = body;
      types = [ty, ...(repeat ?? [])];
    }

    return new TupleType(id, types);
  }),
  path: rule(() => seq(path, astId)).map(
    ([path, id]) => new PathType(id, path),
  ),
};
