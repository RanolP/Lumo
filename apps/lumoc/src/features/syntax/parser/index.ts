import { handsum, type Handsum } from 'handsum';
import type { Parser } from '../../../vendors/malssi/parser';
import { Computation } from '../../ast/computation';
import { Value } from '../../ast/value';
import { RefinedTypeV, TypeC, TypeV } from '../../type';
import { ctx, parser, type ParserInput } from './base';
import { tok } from './token';
import { freshName } from '../../../shared/name';

var ty_c = parser(
  (): Parser<ParserInput, TypeC, void | undefined> =>
    parser.oneOf(
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
          params.reduceRight(
            (result, param) => TypeC.Arrow(param, result),
            body,
          ),
        )
        .labeled('ty_c arrow'),
      parser
        .seq(ty_v.capture('param'), tok.punct.arrow, ty_c.capture('body'))
        .map(({ param, body }) => TypeC.Arrow(param, body))
        .labeled('ty_c arrow'),
      parser
        .seq(tok.punct.paren.l, ty_c.capture('type'), tok.punct.paren.r)
        .map(({ type }) => type)
        .labeled('ty_c paren'),
      parser
        .seq(tok.kw.Produce, ty_v.capture('type'))
        .map(({ type }) => TypeC.Produce(type, {}))
        .labeled('ty_c produce'),
      parser
        .seq(tok.punct.underscore, ctx.freshId.capture('id'))
        .map(({ id }) => TypeC.Variable(`#${id}`))
        .labeled('ty_c variable'),
      parser
        .seq(
          tok.kw.Bundle,
          tok.punct.curly.l,
          parser
            .seq(tok.Tag.capture('name'), tok.punct.colon, ty_c.capture('type'))
            .sepBy(tok.punct.comma)
            .capture('entries'),
          tok.punct.comma.opt(),
          tok.punct.curly.r,
        )
        .map(({ entries }) =>
          TypeC.With(
            Object.fromEntries(entries.map(({ name, type }) => [name, type])),
          ),
        )
        .labeled('ty_c with'),
    ),
).labeled('ty_c');

var ty_v_base = parser(
  (): Parser<ParserInput, RefinedTypeV, void | undefined> =>
    parser.oneOf(
      parser
        .seq(
          tok.punct.mu,
          tok.Ident.capture('name'),
          tok.punct.fullStop,
          ty_v.capture('type'),
        )
        .map(({ name, type }) => TypeV.Recursive(name, type).freshRefined())
        .labeled('ty_v recursive'),
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
        )
        .labeled('ty_v variant'),
      parser
        .seq(
          tok.punct.sum,
          tok.punct.paren.l,
          parser
            .seq(tok.Tag.capture('name'), tok.punct.colon, ty_v.capture('type'))
            .sepBy(tok.punct.comma)
            .capture('entries'),
          tok.punct.comma.opt(),
          tok.punct.paren.r,
        )
        .map(({ entries }) =>
          TypeV.Sum(
            Object.fromEntries(entries.map(({ name, type }) => [name, type])),
          ).freshRefined(),
        )
        .labeled('ty_v sum'),
      parser
        .seq(
          tok.punct.forall,
          tok.Ident.capture('name'),
          tok.punct.fullStop,
          ty_v.capture('type'),
        )
        .map(({ name, type }) => TypeV.TyAbsV(name, type).freshRefined())
        .labeled('ty_v forall_v'),
      parser
        .seq(tok.punct.paren.l, ty_v.capture('type'), tok.punct.paren.r)
        .map(({ type }) => type)
        .labeled('ty_v paren'),
      parser
        .seq(tok.kw.Thunk, ty_c.capture('type'))
        .map(({ type }) => TypeV.Thunk(type).freshRefined())
        .labeled('ty_v thunk'),
      tok.Ident.map((name) => TypeV.Variable(name).freshRefined()).labeled(
        'ty_v variable',
      ),
      parser
        .seq(tok.punct.underscore, ctx.freshId.capture('id'))
        .map(({ id }) => TypeV.Variable(`#${id}`).freshRefined())
        .labeled('ty_v underscore name'),
    ),
).labeled('ty_v_base');

var ty_v = parser(
  (): Parser<ParserInput, RefinedTypeV, void | undefined> =>
    parser
      .seq(
        ty_v_base.capture('base'),
        parser
          .seq(tok.punct.square.l, ty_v.capture('type'), tok.punct.square.r)
          .map(
            ({ type }) =>
              (base: RefinedTypeV) =>
                TypeV.TyAppV(base, type).freshRefined(),
          )
          .repeat()
          .capture('transforms'),
      )
      .map(({ base, transforms }) =>
        transforms.reduce((result, transform) => transform(result), base),
      ),
).labeled('ty_v');

const typedef = parser(() =>
  parser.seq(
    tok.kw.Type,
    tok.Ident.capture('name'),
    tok.punct.equals,
    ty_v.capture('type'),
  ),
).labeled('typedef');

var expr_v_base = parser(
  (): Parser<ParserInput, Value, void | undefined> =>
    parser.oneOf(
      tok.Ident.map((name) => Value.Variable(name)).labeled('expr_v ident'),
      parser
        .seq(tok.punct.paren.l, expr_v.capture('expr'), tok.punct.paren.r)
        .map(({ expr }) => expr)
        .labeled('expr_v paren'),
      parser
        .seq(tok.kw.Roll, expr_v.capture('expr'))
        .map(({ expr }) => Value.Roll(expr)),
      parser
        .seq(tok.kw.Thunk, expr_c.capture('expr'), ctx.freshId.capture('id'))
        .map(({ expr, id }) =>
          Value.Annotate(
            Value.Thunk(expr),
            TypeV.Thunk(TypeC.Variable(`#${id}`)).freshRefined(),
          ),
        ),
      parser
        .seq(tok.kw.Unroll, expr_v.capture('expr'))
        .map(({ expr }) => Value.Unroll(expr)),
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
).labeled('expr_v');

var expr_v = parser(
  (): Parser<ParserInput, Value, void | undefined> =>
    parser
      .seq(
        expr_v_base.capture('base'),
        parser
          .seq(tok.punct.colon, ty_v.capture('type'))
          .map(
            ({ type }) =>
              (base: Value) =>
                Value.Annotate(base, type),
          )
          .repeat()
          .capture('transforms'),
      )
      .map(({ base, transforms }) =>
        transforms.reduce((result, transform) => transform(result), base),
      ),
).labeled('expr_v');

var expr_c_base = parser(
  (): Parser<ParserInput, Computation, void | undefined> =>
    parser.oneOf(
      parser
        .seq(tok.kw.Produce, expr_v.capture('value'), ctx.freshId.capture('id'))
        .map(({ value, id }) =>
          Computation.Annotate(
            Computation.Produce(value),
            TypeC.Produce(TypeV.Variable(`#${id}`).freshRefined(), {}),
          ),
        )
        .labeled('expr_c produce'),
      parser
        .seq(
          tok.Ident.capture('name'),
          tok.punct.leftArrow,
          expr_c.capture('left'),
          tok.punct.semicolon,
          expr_c.capture('right'),
        )
        .map(({ name, left, right }) => Computation.Sequence(left, name, right))
        .labeled('expr_c sequence'),
      parser
        .seq(tok.kw.Force, expr_v.capture('expr'))
        .map(({ expr }) => Computation.Force(expr))
        .labeled('expr_c force'),
      parser
        .seq(tok.punct.paren.l, expr_c.capture('expr'), tok.punct.paren.r)
        .map(({ expr }) => expr)
        .labeled('expr_c paren'),
      fn({ isAnonymous: true }).map(([_name, computation, tyBase]) =>
        Computation.Annotate(computation, tyBase),
      ),
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
        )
        .labeled('expr_c tyapp'),
      parser
        .seq(
          tok.kw.Bundle,
          tok.punct.curly.l,
          parser
            .seq(
              tok.Tag.capture('name'),
              tok.punct.fatArrow,
              expr_c.capture('value'),
              ctx.freshId.capture('id'),
            )
            .sepBy(tok.punct.comma)
            .capture('entries'),
          tok.punct.comma.opt(),
          tok.punct.curly.r,
        )
        .map(({ entries }) =>
          Computation.Annotate(
            Computation.With(
              Object.fromEntries(
                entries.map(({ name, value }) => [name, value]),
              ),
            ),
            TypeC.With(
              Object.fromEntries(
                entries.map(({ name, id }) => [name, TypeC.Variable(`#${id}`)]),
              ),
            ),
          ),
        )
        .labeled('expr_c with'),
      parser
        .seq(
          tok.kw.Match,
          expr_v.capture('value'),
          tok.punct.curly.l,
          parser
            .seq(
              tok.Tag.capture('tag'),
              tok.kw.As,
              tok.Ident.capture('name'),
              tok.punct.fatArrow,
              expr_c.capture('body'),
            )
            .sepBy(tok.punct.comma)
            .capture('branches'),
          tok.punct.comma.opt(),
          tok.punct.curly.r,
        )
        .map(({ value, branches }) =>
          Computation.Match(
            value,
            Object.fromEntries(
              branches.map(({ tag, name, body }) => [tag, [name, body]]),
            ),
          ),
        )
        .labeled('expr_c match'),
      parser
        .seq(
          expr_v.capture('value'),
          tok.punct.fullStop,
          tok.Ident.capture('name'),
        )
        .map(({ value, name }) => Computation.Projection(value, name))
        .labeled('expr_c projection'),
    ),
).labeled('expr_c');

var expr_c = parser(() =>
  parser
    .seq(
      expr_c_base.capture('base'),
      parser
        .oneOf(
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
            ),
          tok.Tag.map(
            (tag) => (base: Computation) => Computation.Resolve(base, tag),
          ),
        )
        .repeat()
        .capture('transforms'),
    )
    .map(({ base, transforms }) =>
      transforms.reduce((result, transform) => transform(result), base),
    ),
).labeled('expr_c');

var fn = ({ isAnonymous }: { isAnonymous: boolean }) =>
  parser
    .seq(
      tok.kw.Fn,
      parser
        .if<string | undefined>(
          isAnonymous,
          parser.noop.map((): string | undefined => undefined),
          tok.Ident.opt(),
        )
        .capture('fnName'),
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
        .seq(tok.Ident.capture('name'), tok.punct.colon, ty_v.capture('type'))
        .sepBy(tok.punct.comma)
        .capture('params'),
      tok.punct.comma.opt(),
      tok.punct.paren.r,
      tok.punct.colon,
      ty_c.capture('returnType'),
      tok.punct.curly.l,
      expr_c.capture('body'),
      tok.punct.curly.r,
    )
    .map(({ typeParams = [], fnName, params, returnType, body }) => {
      const tyBase = params.reduceRight(
        (result, param) => TypeC.Arrow(param.type, result),
        returnType,
      );
      const base = params.reduceRight(
        (result, param) => Computation.Lambda(param.name, result),
        body,
      );
      if (typeParams.length === 0) {
        return [fnName, base, tyBase] as const;
      }
      return [
        fnName,
        Computation.Produce(
          typeParams.reduceRight(
            (result, { name }) => Value.TyAbsV(name, result),
            Value.Thunk(base),
          ),
        ),
        TypeC.Produce(
          typeParams.reduceRight(
            (result, { name }) => TypeV.TyAbsV(name, result).freshRefined(),
            TypeV.Thunk(tyBase).freshRefined(),
          ),
          {},
        ),
      ] as const;
    })
    .labeled('fn');

var enumDef = parser(() =>
  parser.seq(
    tok.kw.Enum,
    parser.cut,
    tok.Ident.capture('name'),
    parser
      .seq(
        tok.punct.square.l,
        parser
          .seq(tok.Ident.capture('name'))
          .sepBy(tok.punct.comma)
          .capture('items'),
        tok.punct.comma.opt(),
        tok.punct.square.r,
      )
      .map(({ items }) => items)
      .opt()
      .capture('typeParams'),
    tok.punct.curly.l,
    parser
      .seq(
        tok.Ident.capture('name'),
        parser
          .seq(
            tok.punct.curly.l,
            parser
              .seq(
                tok.Ident.capture('name'),
                tok.punct.colon,
                ty_v.capture('type'),
              )
              .sepBy(tok.punct.comma)
              .capture('fields'),
            tok.punct.comma.opt(),
            tok.punct.curly.r,
          )
          .map(({ fields }) => fields)
          .opt()
          .capture('fields'),
      )
      .sepBy(tok.punct.comma)
      .capture('variants'),
    tok.punct.comma.opt(),
    tok.punct.curly.r,
    parser.uncut,
  ),
);

export const program = parser(() =>
  parser.seq(
    parser
      .oneOf(
        typedef.map(({ name, type }) => [Item.TypeDef(name, type)]),
        fn({ isAnonymous: false }).map(([name, computation, ty]) => [
          Item.Fn(name!, computation, ty),
        ]),
        enumDef.map(({ name: enumName, typeParams = [], variants }) => {
          // @TODO: typeParams unsupported yet
          const recursionToken = freshName('ty');
          return [
            Item.TypeDef(
              enumName,
              TypeV.Recursive(
                recursionToken,
                TypeV.Sum(
                  Object.fromEntries(
                    variants.map(({ name: variantName, fields = [] }) => [
                      `${enumName}/${variantName}`,
                      TypeV.Variant(
                        `${enumName}/${variantName}`,
                        Object.fromEntries(
                          fields.map(({ name: fieldName, type }) => [
                            fieldName,
                            type.sub_v(
                              enumName,
                              TypeV.Variable(recursionToken).freshRefined(),
                            ),
                          ]),
                        ),
                      ).freshRefined(),
                    ]),
                  ),
                ).freshRefined(),
              ).freshRefined(),
            ),
          ];
        }),
      )
      .repeat()
      .map((items) => items.flat())
      .capture('preamble'),
    expr_c.capture('main'),
  ),
).map(({ preamble, main }) => {
  const { typedefs, fns } = preamble.reduce<{
    typedefs: [string, RefinedTypeV][];
    fns: [string, Computation, TypeC][];
  }>(
    (result, item) => ({
      typedefs: [...result.typedefs, ...(item.TypeDef ? [item.TypeDef] : [])],
      fns: [...result.fns, ...(item.Fn ? [item.Fn] : [])],
    }),
    { typedefs: [], fns: [] },
  );

  const moduleTy = TypeC.With(
    Object.fromEntries(
      preamble.flatMap((item) => {
        if (!item.Fn) return [];
        const [name, _, ty] = item.Fn;
        return [[name, ty]];
      }),
    ),
  );

  const substLetRec = (target: Computation) => {
    for (const [ref] of fns) {
      target = target.sub_v(
        ref,
        Value.Annotate(
          Value.Thunk(
            Computation.Resolve(Computation.Force(Value.Variable('mod')), ref),
          ),
          TypeV.Thunk(TypeC.Variable(freshName('ty'))).freshRefined(),
        ),
      );
    }
    return target;
  };

  return {
    typedefs,
    main: Computation.Def(
      'mod',
      Computation.With(
        Object.fromEntries(
          fns.map(([name, comput]) => {
            return [name, substLetRec(comput)];
          }),
        ),
      ),
      TypeC.With(
        Object.fromEntries(
          preamble.flatMap((item) => {
            if (!item.Fn) return [];
            const [name, _, ty] = item.Fn;
            return [[name, ty]];
          }),
        ),
      ),
      substLetRec(main),
    ),
  };
});

export interface TItem {
  TypeDef(name: string, type: RefinedTypeV): Item;
  Fn(name: string, comput: Computation, ty: TypeC): Item;
}
export type Item = Handsum<TItem>;
export const Item = handsum<TItem>({});
