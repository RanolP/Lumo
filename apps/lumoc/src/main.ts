import { formatParens } from './shared/fmt';
import { Value } from './features/ast/value';
import { TypeC, TypeV } from './features/type';
import { Typer } from './features/typer';
import { dsl } from './features/dsl';
import { Lexer } from './features/syntax/lexer';
import { program } from './features/syntax/parser';
import {
  createArrayInput,
  createContextfulInput,
} from './vendors/malssi/parser/input';

const unit = (() => {
  const t = TypeV.Record({}).freshRefined();
  const v = Value.Record({}).annotate(t);
  return { t, v };
})();
const nat = (() => {
  const zero_t = TypeV.Variant('nat/zero', {}).freshRefined();
  const succ_t = TypeV.Variant('nat/succ', {
    0: TypeV.Variable('X').freshRefined(),
  }).freshRefined();
  const t = TypeV.Recursive(
    'X',
    TypeV.Sum({
      'nat/zero': zero_t,
      'nat/succ': succ_t,
    }).freshRefined(),
  ).freshRefined();

  return {
    t,
    zero: {
      t: zero_t,
      v: Value.Variant('nat/zero', {}).inject('nat/zero').roll().annotate(t),
    },
    succ: {
      t: succ_t,
      v: (expr: Value) =>
        Value.Variant('nat/succ', {
          0: expr,
        })
          .inject('nat/succ')
          .roll()
          .annotate(t),
    },
  };
})();

const tokens = Lexer.lex(
  `
    enum Nat {
      zero,
      succ { n : Nat },
    }
  `,
);
// console.log(tokens);
const items = program.run(
  createContextfulInput({ isBlock: false, id: 0 })(createArrayInput(tokens)),
);
for (const item of items) {
  console.log(formatParens(item.display()));
}

console.log();

const RICH_FORMAT = true;
const fmt = RICH_FORMAT ? formatParens : (source: string) => source;

for (const exprFn of [
  function () {
    return unit.v.annotate(unit.t);
  },
  function () {
    return nat.zero.v;
  },
  function () {
    return nat.succ.v(nat.zero.v);
  },
  function () {
    return nat.succ.v(nat.succ.v(nat.zero.v));
  },
  function () {
    return nat.zero.v.annotate(TypeV.Variable('nat').freshRefined());
  },
  function () {
    return dsl
      .lambda('x', (x) => unit.v.ret())
      .thunk()
      .annotate(TypeC.Arrow(unit.t, unit.t.comput()).thunk().freshRefined());
  },
]) {
  const exprRaw = exprFn.toString();
  const expr = exprFn();
  console.log(
    `src: ${exprRaw.replace(/^function\(\)\{return /, '').replace(/\}$/, '')}`,
  );
  const typer = Typer.create().with('nat', nat.t);
  const typed = typer.infer_v(expr);
}
