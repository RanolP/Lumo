import {
  AstPathType,
  AstTupleType,
  AstType,
  Block,
  DefinitionNode,
  DestructuringPattern,
  DiscardPattern,
  EnumDefinition,
  Expression,
  FunctionCall,
  FunctionDefinition,
  Identifier,
  Match,
  MutName,
  NameBindPattern,
  NameExpression,
  Path,
  Pattern,
} from '@/#core/#ast/index.js';
import {
  Constructor,
  Lambda,
  normalizeType,
  Prod,
  Recursion,
  Sum,
  Type,
  TypeVar,
  TypeScope,
} from '@/#core/#type/index.js';
import { match, P } from 'ts-pattern';
import { TypingError } from '../#core/#type/error.js';
import {
  formatPatternPath,
  PatternDecisionTree,
} from './pattern-decision-tree.js';
import { unify } from './unify.js';

function isSubtypeOf(
  scope: TypeScope,
  subtype: Type,
  supertype: Type,
  goals: [Type, Type][] = [],
): boolean {
  // recursion
  if (goals.some(([l, r]) => l.equals(subtype) && r.equals(supertype))) {
    return true;
  }
  const newGoals: [Type, Type][] = [...goals, [subtype, supertype]];
  const subtypeNorm = normalizeType(scope, subtype);
  const supertypeNorm = normalizeType(scope, supertype);

  // const lca = scope.findLca(left, right);
  // console.log(`LCA(${left}, ${right}) = [${lca.join(', ')}]`);
  // if (lca.some((x) => eq(scope, x, right))) {
  //   return true;
  // }
  return match([subtypeNorm, supertypeNorm])
    .with(
      [P.instanceOf(Constructor), P.instanceOf(Constructor)],
      ([l, r]): boolean =>
        l.folded === r.folded &&
        l.tag === r.tag &&
        match([l, r])
          .with(
            [
              { items: { kind: 'positional' } },
              { items: { kind: 'positional' } },
            ],
            ([l, r]): boolean => {
              if (l.items.types.length !== r.items.types.length) return false;
              for (let i = 0; i < l.items.types.length; i++) {
                if (
                  !isSubtypeOf(
                    scope,
                    l.items.types[i].type,
                    r.items.types[i].type,
                    newGoals,
                  )
                )
                  return false;
              }
              return true;
            },
          )
          .otherwise(() => false),
    )
    .with([P.any, P.instanceOf(Recursion)], ([l, r]): boolean =>
      isSubtypeOf(scope, l, r.unfold(), newGoals),
    )
    .with([P.instanceOf(Recursion), P.any], ([l, r]): boolean =>
      isSubtypeOf(scope, l.unfold(), r, newGoals),
    )
    .with([P.instanceOf(Sum), P.instanceOf(Sum)], ([l, r]): boolean =>
      Array.from(l.items).every((i) => isSubtypeOf(scope, i, r, newGoals)),
    )
    .with([P.any, P.instanceOf(Sum)], ([l, r]): boolean =>
      Array.from(r.items).some((x) => isSubtypeOf(scope, l, x, newGoals)),
    )
    .otherwise((): boolean => {
      return subtypeNorm.equals(supertypeNorm);
    });
}

export function check(
  scope: TypeScope,
  e: Expression | MutName,
  type: Type,
): boolean {
  return match(e)
    .with(P.instanceOf(MutName), (e) => {
      scope.add(e, type);
      return true;
    })
    .otherwise((e) => {
      // Sub<==
      const inferred = infer(scope, e);
      return isSubtypeOf(scope, inferred, type);
    });
}

export function infer(scope: TypeScope, e: Expression): Type {
  return match(e)
    .with(P.instanceOf(FunctionCall), (e): Type => {
      const callScope = scope.createChild(e);
      const fnType = infer(callScope, e.fn);
      type Conversion = { type: Type; description: string | null };
      const conversionStack: Conversion[] = [
        { type: fnType, description: 'the original' },
      ];
      while (true) {
        // @ts-ignore
        let converted = match(conversionStack.at(-1)?.type)
          .with(
            P.instanceOf(TypeVar),
            (ref): Conversion => ({
              type: callScope.lookup(ref.path),
              description: 'scope lookup',
            }),
          )
          .with(P.instanceOf(Recursion), (rec): Conversion => {
            return {
              type: rec.then,
              description: 'recursion unwrap',
            };
          })
          .otherwise(() => null);
        if (converted == null) {
          break;
        }
        conversionStack.push(converted);
      }
      return match(conversionStack.at(-1)?.type)
        .with(P.instanceOf(Lambda), (fnType): Type => {
          if (e.args.length !== fnType.parameters.length) {
            throw new TypingError(
              `Expected ${fnType.parameters.length} arguments but got ${e.args.length}`,
              e,
            );
          }

          for (let i = 0; i < e.args.length; i++) {
            if (!check(scope, e.args[i], fnType.parameters[i])) {
              throw new TypingError(
                `Parameter type ${normalizeType(scope, fnType.parameters[i]).id(
                  scope,
                )} is not compatible with argument type ${match(e.args[i])
                  .with(
                    P.instanceOf(MutName),
                    (arg) => `mut ${arg.ident.token.content}`,
                  )
                  .otherwise((arg) => {
                    try {
                      return normalizeType(scope, infer(scope, arg)).id(scope);
                    } catch {
                      `unknown type`;
                    }
                  })}`,
                e.args[i],
              );
            }
          }

          return fnType.returning;
        })
        .with(P.instanceOf(Constructor), (fnType): Type => {
          throw new TypingError(
            `Found constructor tht is not implemented yet`,
            e,
          );
        })
        .otherwise(() => {
          const typeAndDescription = conversionStack.map(
            ({ type, description }) => [type.toString(), description] as const,
          );
          const typeMaxLength = typeAndDescription.reduce(
            (acc, [ty]) => Math.max(acc, ty.length),
            0,
          );
          throw new TypingError(
            `Expected a function but got: ${typeAndDescription
              .map(
                ([type, description]) =>
                  `${type.padEnd(typeMaxLength, ' ')}${
                    description == null ? '' : `   -- ${description}`
                  }`,
              )
              .join('\n                   which is: ')}`,
            e,
          );
        });
    })
    .with(P.instanceOf(NameExpression), (e): Type => {
      scope.lookup(e); // nonexixstence -> throw
      return new TypeVar(e.id, e.path);
    })
    .with(P.instanceOf(Block), (e): Type => {
      const blockScope = scope.createChild(e);
      let lastType: Type = Prod.unit;
      for (const child of e.expressions) {
        lastType = infer(blockScope, child);
      }
      return lastType;
    })
    .with(P.instanceOf(Match), (e): Type => {
      const exprTy = infer(scope, e.expr);

      const node = new PatternDecisionTree(e.id, e.span);
      for (const arm of e.arms) {
        node.addMatchArm(arm.pattern);
      }

      return node.findMissingPattern(scope, infer(scope, e.expr)).match({
        Ok(_) {
          let result: Type = Sum.never;

          for (const arm of e.arms) {
            const armScope = scope.createChild(arm);
            const introducePatternVars = (pat: Pattern, ty: Type) => {
              match(pat)
                .with(P.instanceOf(DestructuringPattern), (pat) => {
                  match(pat.matches)
                    .with({ type: 'tuple' }, ({ items }) => {
                      let ctor = scope.lookupCtor(pat.destructor.display, arm);
                      if (ctor.items.kind !== 'positional') {
                        throw new TypingError(
                          `Expected positional destructuring but it was not`,
                          pat,
                        );
                      }
                      if (ctor.items.types.length !== items.length) {
                        throw new Error(
                          `Expected ${ctor.items.types.length} items to destruct but got ${items.length}`,
                        );
                      }
                      for (let i = 0; i < items.length; i++) {
                        introducePatternVars(
                          items[i],
                          ctor.items.types[i].type,
                        );
                      }
                    })
                    .exhaustive();
                })
                .with(P.instanceOf(NameBindPattern), (pat) => {
                  armScope.add(pat.name, ty);
                })
                .with(P.instanceOf(DiscardPattern), () => {})
                .exhaustive();
            };

            introducePatternVars(arm.pattern, exprTy);

            result = unify(armScope, result, infer(armScope, arm.body));
          }

          return result;
        },
        Err(errorCase) {
          throw new TypingError(
            `Uncovered pattern: ${formatPatternPath(errorCase)}`,
            e,
          );
        },
      });
    })
    .otherwise(() => {
      throw new TypingError(`Cannot infer expression`, e);
    });
}

export function visit(scope: TypeScope, def: DefinitionNode) {
  match(def)
    .with(P.instanceOf(EnumDefinition), (def) => {
      const constructors = def.branches.map((branch) => {
        const branchScope = scope.createChild(branch);
        branchScope.add(def.name, new TypeVar(def.id, new Path([def.name])));
        return [
          branch,
          match(branch.body)
            .with(
              P.nullish,
              () =>
                new Constructor(
                  branch.id,
                  def.name,
                  branch.name.token.content,
                  { kind: 'positional', types: [] },
                ),
            )
            .with({ kind: 'tuple' }, ({ types }) => {
              const typesMapped = types.map(({ type }) =>
                astTypeToType(branchScope, type),
              );
              return new Constructor(
                branch.id,
                def.name,
                branch.name.token.content,
                {
                  kind: 'positional',
                  types: typesMapped.map((type) => ({ type })),
                },
              );
            })
            .with({ kind: 'struct' }, () => {
              throw new TypingError(
                `Named tuple in enum is not supported yet`,
                branch,
              );
            })
            .exhaustive(),
        ] as const;
      });
      const enumItself = new Recursion(
        def.id,
        new Path([def.name]),
        new Sum(
          def.id,
          constructors.map(([_branch, ty]) => ty),
        ),
      );
      for (const [branch, constructor] of constructors) {
        scope.addCtor(
          new Path([def.name, branch.name]).display,
          constructor.replace(new Path([def.name]), enumItself),
        );
        const enumBodyScope = scope.createChild(branch);
        enumBodyScope.add(def.name, enumItself);

        const name = new Path([def.name, branch.name]);

        scope.add(
          name,
          match(branch.body)
            .with(P.nullish, () => constructor)
            .with({ kind: 'tuple' }, ({ types }) => {
              const typesMapped = types.map(({ type }) =>
                astTypeToType(enumBodyScope, type),
              );
              return new Lambda(branch.id, typesMapped, constructor);
            })
            .with({ kind: 'struct' }, () => {
              throw new TypingError(
                `Struct form enum variant is not supported yet`,
                branch,
              );
            })
            .exhaustive(),
        );
      }

      scope.add(def.name, enumItself);

      for (const [_, ty] of constructors) {
        // should this be fold relation?
        scope.addSubtypeRelation(ty, enumItself);
      }
    })
    .with(P.instanceOf(FunctionDefinition), (def) => {
      const returnType =
        def.returnType != null
          ? astTypeToType(scope, def.returnType)
          : Prod.unit;

      const bodyScope = scope.createChild(def);
      const parameters = [];
      for (const param of def.parameters) {
        if (param.type == null) {
          throw new TypingError(
            `Cannot infer type of param ${param.pattern}`,
            param,
          );
        }
        const ty = astTypeToType(scope, param.type);
        // TODO: pattern support
        if (param.pattern instanceof Identifier) {
          bodyScope.add(param.pattern, ty);
        }
        parameters.push(ty);
      }
      scope.add(def.name, new Lambda(def.id, parameters, returnType));
      check(bodyScope, def.body, returnType);
    })
    .exhaustive();
}

function astTypeToType(scope: TypeScope, ast: AstType): Type {
  return match(ast)
    .with(
      P.instanceOf(AstTupleType),
      (ast) =>
        new Prod(
          ast.id,
          ast.elements.map((ty) => astTypeToType(scope, ty)),
        ),
    )
    .with(P.instanceOf(AstPathType), (ast) => {
      scope.lookup(ast.path); // nonexistence -> throw
      return new TypeVar(ast.id, ast.path);
    })
    .exhaustive();
}
