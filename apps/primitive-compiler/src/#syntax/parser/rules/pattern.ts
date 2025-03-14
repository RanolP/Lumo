import { TokenKind } from '@/#syntax/common/token-kind.js';
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
import { astId, identifier, path, spanning } from './fragments.js';
import {
  DestructuringPattern,
  DiscardPattern,
  NameBindPattern,
  Pattern,
} from '@/#core/#ast/pattern.js';

interface PatternOptions {
  requireLetOnNameBind: boolean;
}
export const pattern = (opts: PatternOptions): Parser<Pattern> =>
  rule(() =>
    oneof(
      'Pattern',
      patterns.destructor(opts),
      patterns.discard,
      patterns.name(opts),
    ),
  );

const patterns = {
  destructor: (opts: PatternOptions) =>
    rule(() =>
      spanning(
        seq(
          path,
          opt(
            seq(
              token(TokenKind.PunctuationLeftParenthesis),
              cut(
                seq(
                  opt(
                    seq(
                      separatedList1(
                        pattern(opts),
                        token(TokenKind.PunctuationComma),
                      ),
                      opt(token(TokenKind.PunctuationComma)),
                    ),
                  ),
                  token(TokenKind.PunctuationRightParenthesis),
                ),
              ),
            ),
          ),
          astId,
        ),
      ),
    ).map(
      ([[destructor, body, id], span]) =>
        new DestructuringPattern(id, span, destructor, {
          type: 'tuple',
          items: body?.[1]?.[0]?.[0] ?? [],
        }),
    ),
  discard: rule(() =>
    spanning(seq(token(TokenKind.IdentifierUnderscore), astId)),
  ).map(([[_, id], span]) => new DiscardPattern(id, span)),
  name: (opts: PatternOptions) =>
    rule(() =>
      spanning(
        opts.requireLetOnNameBind
          ? seq(token(TokenKind.KeywordLet), identifier, astId).map(
              ([_, ident, id]) => [ident, id] as const,
            )
          : seq(identifier, astId),
      ),
    ).map(([[ident, id], span]) => new NameBindPattern(id, span, ident)),
};
