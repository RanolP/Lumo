import { parsecom } from '../../#lib/parsecom';
import { Token, TokenKind } from '../common';

const {
  rule,
  takeIf,
  opt,
  cut,
  seq,
  oneof,
  repeat0,
  separatedList1,
  ctx,
  withCtxMod,
  make,
  failure,
  __Parser,
} = parsecom<
  Token,
  {
    nodeId: number;
    minimumBindingPower: number;
    newlineAsSemi: boolean;
  }
>({
  dropTokenIf(ctx, token) {
    switch (token.kind) {
      case TokenKind.SpaceHorizotanl:
        return true;
      case TokenKind.SpaceVertical:
        return !ctx.newlineAsSemi;
      default:
        return false;
    }
  },
});
export {
  rule,
  takeIf,
  opt,
  cut,
  seq,
  oneof,
  repeat0,
  separatedList1,
  ctx,
  withCtxMod,
  failure,
};

export const token = make.tag(
  (t) => t.kind,
  (token) => token.kind,
);

export type Parser<TOutput> = ReturnType<typeof __Parser<TOutput>>;
