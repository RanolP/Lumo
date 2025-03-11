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
  spanning,
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
  spanning(
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
  ),
).map(
  ([[_0, [name, _1, [body, _2]], id], span]) =>
    new EnumDefinition(id, span, name, body?.[0] ?? []),
);

const enumBranch = rule(() =>
  spanning(
    seq(identifier, opt(oneof('enum branch body', tupleTypeBody)), astId),
  ),
).map(([[name, body, id], span]) => new EnumBranch(id, span, name, body));

const fn = rule(() =>
  spanning(
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
  ),
).map(([[_0, [name, _1, params, _2, returnType, body], id], span]) => {
  return new FunctionDefinition(
    id,
    span,
    name,
    params?.[0] ?? [],
    returnType?.[1] ?? null,
    body,
  );
});

const fnParam = rule(() =>
  spanning(
    seq(
      fnParamPattern,
      opt(seq(token(TokenKind.PunctuationColon), type)),
      astId,
    ),
  ),
).map(([[pat, ty, id], span]) => new FunctionParameter(id, span, pat, ty?.[1]));

const fnParamPattern = rule(() =>
  oneof('Function Parameter Pattern', identifier),
);

export const definitions = {
  enum: enumRule,
  fn,
};
