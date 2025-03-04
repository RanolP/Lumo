import {
  cut,
  failure,
  oneof,
  opt,
  rule,
  separatedList1,
  seq,
  token,
} from '../base.js';
import { TokenKind } from '../../common/index.js';
import {
  astId,
  identifier,
  tupleTypeBody,
  withoutNewlineAsSemi,
} from './fragments.js';
import {
  EnumBranch,
  EnumDefinition,
  FunctionDefinition,
  FunctionParameter,
} from '@/#core/#ast/index.js';
import { expression, expressions } from './expression/index.js';
import { type } from './type.js';

export const definition = rule(() =>
  oneof('Definition', definitions.enum, definitions.fn),
);

const enumRule = rule(() =>
  seq(
    token(TokenKind.KeywordEnum),
    cut(
      seq(
        identifier,
        token(TokenKind.PunctuationLeftCurlyBracket),
        withoutNewlineAsSemi(
          seq(
            opt(
              seq(
                separatedList1(enumBranch, token(TokenKind.PunctuationComma)),
                opt(token(TokenKind.PunctuationComma)),
              ),
            ),
            token(TokenKind.PunctuationRightCurlyBracket),
          ),
        ),
      ),
    ),
    astId,
  ),
).map(
  ([_0, [name, _1, [body, _2]], id]) =>
    new EnumDefinition(id, name, body?.[0] ?? []),
);

const enumBranch = rule(() =>
  seq(identifier, opt(oneof('enum branch body', tupleTypeBody))),
).map(([name, body]) => new EnumBranch(name));

const fn = rule(() =>
  seq(
    token(TokenKind.KeywordFn),
    cut(
      seq(
        identifier,
        token(TokenKind.PunctuationLeftParenthesis),
        opt(
          seq(
            separatedList1(fnParam, token(TokenKind.PunctuationComma)),
            opt(token(TokenKind.PunctuationComma)),
          ),
        ),
        token(TokenKind.PunctuationRightParenthesis),
        opt(seq(token(TokenKind.PunctuationColon), type)),
        oneof(
          'Function Body',
          seq(token(TokenKind.PunctuationsFatArrow), expression).map(
            ([_, expr]) => expr,
          ),
          expressions.block,
        ),
      ),
    ),
    astId,
  ),
).map(([_0, [name, _1, params, _2, returnType, body], id]) => {
  return new FunctionDefinition(
    id,
    name,
    params?.[0] ?? [],
    returnType?.[1] ?? null,
    body,
  );
});

const fnParam = rule(() =>
  seq(fnParamPattern, opt(seq(token(TokenKind.PunctuationColon), type)), astId),
).map(([pat, ty, id]) => new FunctionParameter(id, pat, ty?.[1]));

const fnParamPattern = rule(() =>
  oneof('Function Parameter Pattern', identifier),
);

export const definitions = {
  enum: enumRule,
  fn,
};
