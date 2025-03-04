import { AstId, Identifier, Path } from '@/#core/#ast/index.js';
import { TokenKind } from '../../common/index.js';
import {
  ctx,
  cut,
  opt,
  rule,
  separatedList1,
  seq,
  token,
  withCtxMod,
} from '../base.js';
import { type } from './type.js';

export const identifier = rule(() => token(TokenKind.Identifier)).map(
  (token) => new Identifier(token),
);

export const path = rule(() =>
  separatedList1(identifier, token(TokenKind.PunctuationFullStop)).map(
    (seguments) => new Path(seguments),
  ),
);

export const tupleTypeBody = rule(() =>
  seq(
    token(TokenKind.PunctuationLeftParenthesis),
    withoutNewlineAsSemi(
      cut(
        seq(
          opt(
            seq(
              separatedList1(type, token(TokenKind.PunctuationComma)),
              opt(token(TokenKind.PunctuationComma)),
            ),
          ),
          token(TokenKind.PunctuationRightParenthesis),
        ),
      ),
    ),
  ),
).map(([_0, [types, _1]]) => types?.[0] ?? []);

export const astId = rule(() =>
  ctx(({ nodeId, ...rest }) => [{ nodeId: nodeId + 1, ...rest }, nodeId + 1]),
).map((id) => new AstId(id));

export const withNewlineAsSemi = withCtxMod('newlineAsSemi', true);
export const withoutNewlineAsSemi = withCtxMod('newlineAsSemi', false);
