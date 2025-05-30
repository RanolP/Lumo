import {
  AstNode,
  Block,
  EnumDefinition,
  Expression,
  FunctionDefinition,
  Identifier,
  Match,
  NameExpression,
  Path,
} from '@/#core/#ast/index.js';
import { Lambda, Type, TypeVar } from '@/#core/#type/variants.js';
import {
  Assignment,
  Fn,
  Seq,
  TodoExpr,
  TsExpr,
  VarName,
} from '@/#lib/simple-ts-ast/expr.js';
import { Define, TsAst } from '@/#lib/simple-ts-ast/index.js';
import { TsTy, Typename } from '@/#lib/simple-ts-ast/ty.js';
import { match, P } from 'ts-pattern';
import { CompileContext } from './compile-context.js';

export function compile(ctx: CompileContext, node: AstNode): TsAst {
  return match(node)
    .with(P.instanceOf(EnumDefinition), (node) => {
      return [];
      // throw new Error('Not implemented: compile EnumDefinition');
    })
    .with(P.instanceOf(FunctionDefinition), (node) => {
      ctx = ctx.of(node);
      const ty = ctx.scope.lookup(node.name);

      if (!(ty instanceof Lambda)) {
        throw new Error('Expected a function type');
      }

      return new Define(
        node.name.token.content,
        new Fn(
          ty.parameters.map((param, index) => [
            `p${index}`,
            compileType(ctx, param),
          ]),
          compileType(ctx, ty.returning),
          new Seq([
            ...node.parameters.map((param, index) =>
              match(param.pattern)
                .with(
                  P.instanceOf(Identifier),
                  (ident) =>
                    new Assignment(
                      ident.token.content,
                      new VarName(`p${index}`),
                    ),
                )
                .otherwise(() => {
                  throw new Error('Not implemented: compile parameter');
                }),
            ),
            compileExpr(ctx, node.body),
          ]),
        ),
      );
    })
    .exhaustive();
}

export function compileExpr(ctx: CompileContext, expr: Expression): TsExpr {
  return match(expr)
    .with(
      P.instanceOf(Block),
      (expr) =>
        new Seq(expr.expressions.map((child) => compileExpr(ctx, child))),
    )
    .with(P.instanceOf(Match), (expr): TsExpr => {
      const variable = ctx.generateUniqueVariable();
      return new Seq([
        new Assignment(variable, compileExpr(ctx, expr.expr)),
        TodoExpr,
      ]);
    })
    .with(
      P.instanceOf(NameExpression),
      (expr): TsExpr => new VarName(expr.path.display),
    )
    .otherwise(() => {
      throw new Error(`Not implemented: compileExpr(${expr})`);
    });
}

export function compileType(ctx: CompileContext, ty: Type): TsTy {
  return match(ty)
    .with(P.instanceOf(TypeVar), (ty) => new Typename(ty.path.display))
    .otherwise(() => {
      throw new Error('Not implemented: compileType(' + ty + ')');
    });
}
