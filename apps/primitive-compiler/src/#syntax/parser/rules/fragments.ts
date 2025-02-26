import { AstId } from '../../../#core/#ast/base';
import { Identifier } from '../../../#core/#ast';
import { TokenKind } from '../../common';
import {
  ctx,
  cut,
  opt,
  rule,
  separatedList1,
  seq,
  token,
  withCtxMod,
} from '../base';
import { type } from './type';

export const identifier = rule(() => token(TokenKind.Identifier)).map(
  (token) => new Identifier(token),
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
