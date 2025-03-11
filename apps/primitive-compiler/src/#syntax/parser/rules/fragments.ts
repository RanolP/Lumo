import { AstId, Identifier, Path } from '@/#core/#ast/index.js';
import { Span, TokenKind } from '../../common/index.js';
import {
  ctx,
  cut,
  opt,
  Parser,
  peek,
  rule,
  separatedList1,
  seq,
  token,
  withCtxMod,
} from '../base.js';
import { type } from './type.js';

export const spanning = <TOutput>(parser: Parser<TOutput>) =>
  rule(() =>
    seq(
      peek().map((t) => t.span.begin),
      parser,
      peek().map((t) => t.span.begin),
    ).map(([begin, value, end]) => [value, Span.make({ begin, end })] as const),
  );

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
).map(
  ([_0, [types, _1]]) =>
    ({
      kind: 'tuple',
      types: (types?.[0] ?? []).map((type) => ({ type })),
    } as const),
);

export const astId = rule(() =>
  ctx(({ nodeId, ...rest }) => [{ nodeId: nodeId + 1, ...rest }, nodeId + 1]),
).map((id) => new AstId(id));

export const withNewlineAsSemi = withCtxMod('newlineAsSemi', true);
export const withoutNewlineAsSemi = withCtxMod('newlineAsSemi', false);
