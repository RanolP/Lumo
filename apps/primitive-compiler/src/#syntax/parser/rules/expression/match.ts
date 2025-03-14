import { TokenKind } from '@/#syntax/common/token-kind.js';
import {
  cut,
  failure,
  opt,
  rule,
  separatedList1,
  seq,
  token,
  withCtxMod,
} from '../../base.js';
import { expression } from './index.js';
import { astId, spanning } from '../fragments.js';
import { Match, MatchArm } from '@/#core/#ast/expression/index.js';
import { pattern } from '../pattern.js';

export const matchExpr = rule(() =>
  spanning(
    seq(
      token(TokenKind.KeywordMatch),
      cut(
        seq(
          expression,
          token(TokenKind.PunctuationLeftCurlyBracket),
          withCtxMod(
            'newlineAsSemi',
            false,
          )(
            opt(
              seq(
                separatedList1(matchArm, token(TokenKind.PunctuationComma)),
                opt(token(TokenKind.PunctuationComma)),
              ),
            ),
          ),
          token(TokenKind.PunctuationRightCurlyBracket),
          astId,
        ),
      ),
    ),
  ),
).map(
  ([[_0, [expr, _1, arms, _2, id]], span]) =>
    new Match(id, span, expr, arms?.[0] ?? []),
);

const matchArm = rule(() =>
  spanning(
    seq(
      pattern({ requireLetOnNameBind: true }),
      token(TokenKind.PunctuationsFatArrow),
      expression,
      astId,
    ),
  ),
).map(
  ([[pattern, _, body, id], span]) => new MatchArm(id, span, pattern, body),
);
