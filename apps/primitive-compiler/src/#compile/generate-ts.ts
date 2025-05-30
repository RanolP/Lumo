import {
  Assignment,
  Fn,
  Seq,
  TodoExpr,
  TsExpr,
  VarName,
} from '@/#lib/simple-ts-ast/expr.js';
import { Define, ExprStmt, TsAst } from '@/#lib/simple-ts-ast/index.js';
import { TsTy, Typename } from '@/#lib/simple-ts-ast/ty.js';
import { match, P } from 'ts-pattern';

export function generateTs(node: TsAst): string {
  return match(node)
    .with(P.array(P.any), (node) =>
      node.map((child) => generateTs(child)).join('\n\n'),
    )
    .with(
      P.instanceOf(Define),
      (node) => `const ${node.name} = ${generateTsExpr(node.expr)};`,
    )
    .with(P.instanceOf(ExprStmt), (stmt) => generateTsExpr(stmt.expr))
    .exhaustive();
}

export function generateTsExpr(node: TsExpr): string {
  return match(node)
    .with(
      P.instanceOf(Fn),
      (node) =>
        `(${node.params
          .map(([name, ty]) => `${name}: ${generateTsType(ty)}`)
          .join(', ')}): ${generateTsType(node.returnType)} => ${generateTsExpr(
          node.body,
        )}`,
    )
    .with(P.instanceOf(Seq), (node) =>
      node.exprs.map(generateTsExpr).join(', '),
    )
    .with(
      P.instanceOf(Assignment),
      (node) => `${node.name} = (${generateTsExpr(node.value)})`,
    )
    .with(P.instanceOf(VarName), (node) => node.name)
    .with(TodoExpr, () => 'todo()')
    .otherwise(() => {
      throw new Error(`Not implemented: generateTsExpr(${String(node)})`);
    });
}

export function generateTsType(node: TsTy): string {
  return match(node)
    .with(P.instanceOf(Typename), (node) => node.name)
    .exhaustive();
}
