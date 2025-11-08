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
import { Computation } from './features/ast/computation';
import { freshName } from './shared/name';

const unit = (() => {
  const t = TypeV.Record({}).freshRefined();
  const v = Value.Record({}).annotate(t);
  return { t, v };
})();
const nat = (() => {
  const recursionName = freshName();
  const zero_t = TypeV.Variant('nat/zero', {}).freshRefined();
  const succ_t = TypeV.Variant('nat/succ', {
    0: TypeV.Variable(recursionName).freshRefined(),
  }).freshRefined();
  const t = TypeV.Recursive(
    recursionName,
    TypeV.Sum({
      'nat/zero': zero_t,
      'nat/succ': succ_t,
    }).freshRefined(),
  ).freshRefined();

  return {
    t,
    zero: {
      t: zero_t,
      tag: 'nat/zero',
      v: Value.Variant('nat/zero', {}).inject('nat/zero').roll().annotate(t),
    },
    succ: {
      t: succ_t,
      tag: 'nat/succ',
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
const maybe_nat = (() => {
  const recursionName = freshName();
  const nothing_t = TypeV.Variant('maybe_nat/nothing', {}).freshRefined();
  const just_t = TypeV.Variant('maybe_nat/just', {
    value: nat.t,
  }).freshRefined();
  const t = TypeV.Recursive(
    recursionName,
    TypeV.Sum({
      'maybe_nat/nothing': nothing_t,
      'maybe_nat/just': just_t,
    }).freshRefined(),
  ).freshRefined();
  return {
    t,
    nothing: {
      t: nothing_t,
      tag: 'maybe_nat/nothing',
      v: Value.Variant('maybe_nat/nothing', {})
        .inject('maybe_nat/nothing')
        .roll()
        .annotate(t),
    },
    just: {
      t: just_t,
      tag: 'maybe_nat/just',
      v: (expr: Value) =>
        Value.Variant('maybe_nat/just', { value: expr })
          .inject('maybe_nat/just')
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
// for (const item of items) {
//   console.log(formatParens(item.display()));
// }

// console.log();

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
    return maybe_nat.nothing.v;
  },
  function () {
    return maybe_nat.just.v(nat.zero.v);
  },
  function () {
    return maybe_nat.just.v(nat.succ.v(nat.zero.v));
  },
  function () {
    return dsl
      .lambda((x) => unit.v.ret())
      .thunk()
      .annotate(TypeC.Arrow(unit.t, unit.t.comput()).thunk().freshRefined());
  },
  function () {
    return dsl
      .lambda((x) =>
        Computation.Match(Value.Unroll(x), {
          [nat.zero.tag]: dsl.matchArm((x) =>
            maybe_nat.nothing.v.ret().annotate(maybe_nat.t.comput()),
          ),
          [nat.succ.tag]: dsl.matchArm((x) =>
            dsl.bind(x.select('0'), 'y', (y) =>
              maybe_nat.just.v(y).ret().annotate(maybe_nat.t.comput()),
            ),
          ),
        }),
      )
      .thunk()
      .annotate(
        TypeC.Arrow(nat.t, maybe_nat.t.comput()).thunk().freshRefined(),
      );
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
