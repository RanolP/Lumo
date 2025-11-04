import { Computation } from '../../ast/computation';
import { Value } from '../../ast/value';
import { TypeV } from '../../type';
import { ctx, parser } from './base';
import { Item } from './item';
import { tok } from './token';

const frag = {
  get namedtuple() {
    return parser.seq(
      tok.punct.curly.l,
      parser
        .seq(
          tok.Ident.capture('name'),
          tok.punct.colon,
          frag.type.capture('type'),
        )
        .sepBy(tok.punct.comma)
        .capture('fields'),
      tok.punct.curly.r,
    );
  },
  type: tok.Ident.map((name) => TypeV.Variable(name).freshRefined()),
};

const def = {
  enum: parser
    .seq(
      tok.kw.Enum,
      ctx.freshId.capture('id'),
      tok.Ident.capture('name'),
      tok.punct.curly.l,
      parser
        .seq(
          parser
            .seq(
              tok.Ident.capture('name'),
              frag.namedtuple.opt().capture('body'),
            )
            .sepBy(tok.punct.comma, 1)
            .capture('items'),
          tok.punct.comma.opt(),
        )
        .opt()
        .capture('items'),
      tok.punct.curly.r,
    )
    .map(({ id, name, items: { items = [] } = {} }) => [
      Item.LetType(
        name,
        TypeV.Recursive(
          `#${id}`,
          TypeV.Sum(
            Object.fromEntries(
              items.map(({ name, body: { fields = [] } = {} }) => [
                name,
                TypeV.Variant(
                  name,
                  Object.fromEntries(
                    fields.map(({ name, type }) => [name, type]),
                  ),
                ).freshRefined(),
              ]),
            ),
          )
            .freshRefined()
            .sub(name, TypeV.Variable(`#${id}`).freshRefined()),
        ).freshRefined(),
      ),
      Item.LetComputation(
        name,
        Computation.With(
          Object.fromEntries(
            items.map((item) => [
              item.name,
              Computation.Return(
                Value.Variant(item.name, {}).inject(item.name).roll(),
              ),
            ]),
          ),
        ),
      ),
    ]),
};

export const program = def.enum;
