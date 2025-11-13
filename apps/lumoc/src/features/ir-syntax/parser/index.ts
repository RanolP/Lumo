import { freshName } from '../../../shared/name';
import { Parser } from '../../../vendors/malssi/parser';
import { Computation } from '../../ast/computation';
import { Value } from '../../ast/value';
import { RefinedTypeV, TypeC, TypeV } from '../../type';
import { ctx, parser, ParserInput } from './base';
import { tok } from './token';

var ty_c = parser(
  (): Parser<ParserInput, TypeC, void | undefined> =>
    parser
      .seq(
        tok.punct.paren.l,
        ty_v.sepBy(tok.punct.comma).capture('params'),
        tok.punct.comma.opt(),
        tok.punct.paren.r,
        tok.punct.arrow,
        ty_c.capture('body'),
      )
      .map(({ params, body }) =>
        params.reduceRight((result, param) => TypeC.Arrow(param, result), body),
      )
      .or(
        parser
          .seq(ty_v.capture('param'), tok.punct.arrow, ty_c.capture('body'))
          .map(({ param, body }) => TypeC.Arrow(param, body)),
      )
      .or(
        parser
          .seq(tok.punct.paren.l, ty_c.capture('type'), tok.punct.paren.r)
          .map(({ type }) => type),
      )
      .or(
        parser
          .seq(tok.kw.Produce, ty_v.capture('type'))
          .map(({ type }) => TypeC.Produce(type, {})),
      ),
);

var ty_v = parser(
  (): Parser<ParserInput, RefinedTypeV, void | undefined> =>
    parser
      .seq(
        tok.punct.mu,
        tok.Ident.capture('name'),
        tok.punct.fullStop,
        ty_v.capture('type'),
      )
      .map(({ name, type }) => TypeV.Recursive(name, type).freshRefined())
      .or(
        parser
          .seq(
            tok.Tag.capture('tag'),
            tok.punct.curly.l,
            parser
              .seq(
                tok.Ident.capture('name'),
                tok.punct.colon,
                ty_v.capture('type'),
              )
              .sepBy(tok.punct.comma)
              .capture('entries'),
            tok.punct.comma.opt(),
            tok.punct.curly.r,
          )
          .map(({ tag, entries }) =>
            TypeV.Variant(
              tag,
              Object.fromEntries(entries.map(({ name, type }) => [name, type])),
            ).freshRefined(),
          ),
      )
      .or(
        parser
          .seq(
            tok.punct.sum,
            tok.punct.paren.l,
            parser
              .seq(
                tok.Tag.capture('name'),
                tok.punct.colon,
                ty_v.capture('type'),
              )
              .sepBy(tok.punct.comma)
              .capture('entries'),
            tok.punct.comma.opt(),
            tok.punct.paren.r,
          )
          .map(({ entries }) =>
            TypeV.Sum(
              Object.fromEntries(entries.map(({ name, type }) => [name, type])),
            ).freshRefined(),
          ),
      )
      .or(
        parser
          .seq(
            tok.punct.forall,
            tok.Ident.capture('name'),
            tok.punct.fullStop,
            ty_v.capture('type'),
          )
          .map(({ name, type }) => TypeV.TyAbsV(name, type).freshRefined()),
      )
      .or(
        parser
          .seq(tok.punct.paren.l, ty_v.capture('type'), tok.punct.paren.r)
          .map(({ type }) => type),
      )
      .or(
        parser
          .seq(tok.kw.Thunk, ty_c.capture('type'))
          .map(({ type }) => TypeV.Thunk(type).freshRefined()),
      )
      .or(tok.Ident.map((name) => TypeV.Variable(name).freshRefined()))
      .or(
        parser
          .seq(tok.punct.underscore, ctx.freshId.capture('id'))
          .map(({ id }) => TypeV.Variable(`#${id}`).freshRefined()),
      ),
);

const typedef = parser(() =>
  parser.seq(
    tok.kw.Type,
    tok.Ident.capture('name'),
    tok.punct.equals,
    ty_v.capture('type'),
  ),
);

var expr_v = parser(
  (): Parser<ParserInput, Value, void | undefined> =>
    tok.Ident.map((name) => Value.Variable(name))
      .or(
        parser
          .seq(tok.punct.paren.l, expr_v.capture('expr'), tok.punct.paren.r)
          .map(({ expr }) => expr),
      )
      .or(
        parser
          .seq(tok.kw.Roll, expr_v.capture('expr'))
          .map(({ expr }) => Value.Roll(expr)),
      )
      .or(
        parser
          .seq(tok.kw.Thunk, expr_c.capture('expr'))
          .map(({ expr }) => Value.Thunk(expr)),
      ),
);

var expr_c_base = parser(
  (): Parser<ParserInput, Computation, void | undefined> =>
    parser
      .seq(tok.kw.Produce, expr_v.capture('value'))
      .map(({ value }) => Computation.Produce(value))
      .or(
        parser
          .seq(
            tok.Ident.capture('name'),
            tok.punct.leftArrow,
            expr_c.capture('left'),
            tok.punct.semicolon,
            expr_c.capture('right'),
          )
          .map(({ name, left, right }) =>
            Computation.Sequence(left, name, right),
          ),
      )
      .or(
        parser
          .seq(tok.kw.Force, expr_v.capture('expr'))
          .map(({ expr }) => Computation.Force(expr)),
      )
      .or(
        parser
          .seq(tok.punct.paren.l, expr_c.capture('expr'), tok.punct.paren.r)
          .map(({ expr }) => expr),
      )
      .or(fn),
);

var expr_c = parser(() =>
  parser
    .seq(
      expr_c_base.capture('base'),
      parser
        .seq(
          tok.punct.paren.l,
          expr_v.sepBy(tok.punct.comma).capture('values'),
          tok.punct.comma.opt(),
          tok.punct.paren.r,
        )
        .map(
          ({ values }) =>
            (base: Computation) =>
              values.reduce(
                (result, value) => Computation.Apply(result, value),
                base,
              ),
        )
        .repeat()
        .capture('transforms'),
    )
    .map(({ base, transforms }) =>
      transforms.reduce((result, transform) => transform(result), base),
    ),
);

var fn = parser
  .seq(
    tok.kw.Fn,
    tok.Ident.opt(),
    parser
      .seq(
        tok.punct.square.l,
        tok.Ident.sepBy(tok.punct.comma).capture('items'),
        tok.punct.comma.opt(),
        tok.punct.square.r,
      )
      .map(({ items }) => items)
      .opt()
      .capture('typeParams'),
    tok.punct.paren.l,
    parser
      .seq(tok.Ident.capture('name'), tok.punct.colon, ty_v.capture('type'))
      .sepBy(tok.punct.comma)
      .capture('params'),
    tok.punct.comma.opt(),
    tok.punct.paren.r,
    tok.punct.colon,
    ty_c.capture('returnType'),
    tok.punct.fatArrow,
    expr_c.capture('body'),
  )
  .map(({ typeParams = [], params, returnType, body }) => {
    const expr = typeParams.reduceRight(
      (result, name) =>
        Computation.Produce(Value.TyAbsV(name, Value.Thunk(result))),
      params.reduceRight(
        (result, param) => Computation.Lambda(param.name, result),
        body,
      ),
    );
    const ty = typeParams.reduceRight(
      (result, name) =>
        TypeC.Produce(
          TypeV.TyAbsV(name, TypeV.Thunk(result).freshRefined()).freshRefined(),
          {},
        ),
      params.reduceRight(
        (result, param) => TypeC.Arrow(param.type, result),
        returnType,
      ),
    );
    return expr.annotate(ty);
  });

export const program = parser(() =>
  parser.seq(typedef.repeat().capture('typedefs'), expr_c.capture('main')),
);
