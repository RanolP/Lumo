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
      )
      .or(
        parser
          .seq(tok.punct.underscore, ctx.freshId.capture('id'))
          .map(({ id }) => TypeC.Variable(`#${id}`)),
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
      )
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
          .seq(tok.kw.Thunk, expr_c.capture('expr'), ctx.freshId.capture('id'))
          .map(({ expr, id }) =>
            Value.Annotate(
              Value.Thunk(expr),
              TypeV.Thunk(TypeC.Variable(`#${id}`)).freshRefined(),
            ),
          ),
      )
      .or(
        parser
          .seq(tok.kw.Unroll, expr_v.capture('expr'))
          .map(({ expr }) => Value.Unroll(expr)),
      )
      .or(
        parser
          .seq(
            tok.Tag.capture('tag'),
            tok.punct.curly.l,
            parser
              .seq(
                tok.Ident.capture('name'),
                tok.punct.colon,
                expr_v.capture('value'),
              )
              .sepBy(tok.punct.comma)
              .capture('entries'),
            tok.punct.comma.opt(),
            tok.punct.curly.r,
          )
          .map(({ tag, entries }) =>
            Value.Injection(
              tag,
              Value.Variant(
                tag,
                Object.fromEntries(
                  entries.map(({ name, value }) => [name, value]),
                ),
              ),
            ),
          ),
      ),
);

var expr_c_base = parser(
  (): Parser<ParserInput, Computation, void | undefined> =>
    parser
      .seq(tok.kw.Produce, expr_v.capture('value'), ctx.freshId.capture('id'))
      .map(({ value, id }) =>
        Computation.Annotate(
          Computation.Produce(value),
          TypeC.Produce(TypeV.Variable(`#${id}`).freshRefined(), {}),
        ),
      )
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
      .or(fn)
      .or(
        parser
          .seq(
            expr_v.capture('base'),
            tok.punct.square.l,
            parser
              .seq(ty_v.capture('type'), ctx.freshId.capture('id'))
              .sepBy(tok.punct.comma)
              .capture('types'),
            tok.punct.comma.opt(),
            tok.punct.square.r,
            ctx.freshId.capture('exprId'),
          )
          .map(({ base, types, exprId }) =>
            types.reduce(
              (result, type) =>
                Computation.Sequence(
                  result,
                  `#${type.id}`,
                  Computation.TyAppV(base, type.type),
                ),
              Computation.Annotate(
                Computation.Produce(base),
                TypeC.Produce(TypeV.Variable(`#${exprId}`).freshRefined(), {}),
              ),
            ),
          ),
      ),
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
        parser
          .seq(tok.Ident.capture('name'), ctx.freshId.capture('id'))
          .sepBy(tok.punct.comma)
          .capture('items'),
        tok.punct.comma.opt(),
        tok.punct.square.r,
      )
      .map(({ items }) => items)
      .opt()
      .capture('typeParams'),
    tok.punct.paren.l,
    parser
      .seq(
        tok.Ident.capture('name'),
        tok.punct.colon,
        ty_v.capture('type'),
        ctx.freshId.capture('id'),
      )
      .sepBy(tok.punct.comma)
      .capture('params'),
    tok.punct.comma.opt(),
    tok.punct.paren.r,
    tok.punct.colon,
    ty_c.capture('returnType'),
    tok.punct.curly.l,
    expr_c.capture('body'),
    tok.punct.curly.r,
    ctx.freshId.capture('exprId1'),
    ctx.freshId.capture('exprId2'),
  )
  .map(({ typeParams = [], params, returnType, body, exprId1, exprId2 }) => {
    const base = params.reduceRight(
      (result, param) =>
        Computation.Annotate(
          Computation.Lambda(param.name, result),
          TypeC.Arrow(param.type, TypeC.Variable(`#${param.id}`)),
        ),
      Computation.Annotate(body, returnType),
    );
    if (typeParams.length === 0) {
      return base;
    }
    return Computation.Annotate(
      Computation.Produce(
        typeParams.reduceRight(
          (result, { name, id }) =>
            Value.Annotate(
              Value.TyAbsV(name, result),
              TypeV.TyAbsV(
                name,
                TypeV.Variable(`#${id}`).freshRefined(),
              ).freshRefined(),
            ),
          Value.Annotate(
            Value.Thunk(base),
            TypeV.Thunk(TypeC.Variable(`#${exprId1}`)).freshRefined(),
          ),
        ),
      ),
      TypeC.Produce(TypeV.Variable(`#${exprId2}`).freshRefined(), {}),
    );
  });

export const program = parser(() =>
  parser.seq(typedef.repeat().capture('typedefs'), expr_c.capture('main')),
);
