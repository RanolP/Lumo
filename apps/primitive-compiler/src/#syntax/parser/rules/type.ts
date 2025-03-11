import { AstPathType, AstTupleType, AstType } from '@/#core/#ast/index.js';
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
import { astId, path, spanning } from './fragments.js';

export const type: Parser<AstType> = rule(() =>
  oneof('type', types.tuple, types.path),
);

const types = {
  tuple: rule(() =>
    spanning(
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
    ),
  ).map(([[_0, [body, _1], id], span]) => {
    let types: AstType[] = [];
    if (body != null) {
      const [ty, _2, repeat, _3] = body;
      types = [ty, ...(repeat ?? [])];
    }

    return new AstTupleType(id, span, types);
  }),
  path: rule(() => spanning(seq(path, astId))).map(
    ([[path, id], span]) => new AstPathType(id, span, path),
  ),
};
