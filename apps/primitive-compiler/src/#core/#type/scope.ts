import {
  AstId,
  Expression,
  Identifier,
  MutName,
  NameExpression,
  Path,
} from '@/#core/#ast/index.js';
import { Constructor, Type } from '@/#core/#type/index.js';
import { TypingError } from './error.js';
import { match, P } from 'ts-pattern';
import { IAstNode } from '@/#core/#ast/base.js';
import { findLowestCommonAncestor, TypeGraph } from './subtyping-dag-lca.js';

export class TypeScope {
  private astTypeMapping = new Map<number | string, Type>();
  private idTypeMapping = new Map<string, Type>();
  private nameCtorMapping = new Map<string, Constructor>();
  private subtypingRelation: TypeGraph = new Map();
  private children = new Map<number, TypeScope>();

  constructor(private readonly parent: TypeScope | null = null) {}

  createChild(node: IAstNode): TypeScope {
    const child = new TypeScope(this);
    this.children.set(node.id.handle, child);
    return child;
  }

  of(node: IAstNode): TypeScope {
    return this.children.get(node.id.handle) ?? this;
  }

  add(expr: Expression | MutName | Path | Identifier, ty: Type) {
    const key = extractKey(expr);
    /// we have no unify.
    if (this.astTypeMapping.has(key)) {
      throw new TypingError(
        `Incompatible type set for ${expr.toString()}`,
        null,
      );
    }
    this.astTypeMapping.set(key, ty);
    this.idTypeMapping.set(ty.id(this), ty);
  }

  addCtor(name: string, ty: Constructor) {
    if (this.nameCtorMapping.has(name)) {
      throw new TypingError(`Constructor ${name} is already registered`, null);
    }
    this.nameCtorMapping.set(name, ty);
  }

  lookupOrNull(expr: Expression | Identifier | Path): Type | null {
    return (
      this.astTypeMapping.get(extractKey(expr)) ??
      this.parent?.lookup(expr) ??
      null
    );
  }

  lookupCtorOrNull(name: string): Constructor | null {
    return (
      this.nameCtorMapping.get(name) ??
      this.parent?.lookupCtorOrNull(name) ??
      null
    );
  }

  lookup(expr: Expression | Identifier | Path): Type {
    const result = this.lookupOrNull(expr);
    if (result == null) {
      throw new TypingError(
        `Cannot resolve type for ${extractKey(expr)} in scope`,
        expr,
      );
    }
    return result;
  }

  lookupCtor(name: string, expr: IAstNode): Constructor {
    const result = this.lookupCtorOrNull(name);
    if (result == null) {
      throw new TypingError(`Unknown constructor ${name} in scope`, expr);
    }
    return result;
  }

  // unused now
  addSubtypeRelation(a: Type, b: Type) {
    const aId = a.id(this);
    const bId = b.id(this);
    this.subtypingRelation.set(aId, [
      bId,
      ...(this.subtypingRelation.get(aId) ?? []),
    ]);
  }

  // unused now
  findLca(a: Type, b: Type): Type[] {
    return Array.from(
      findLowestCommonAncestor(this.subtypingRelation, a.id(this), b.id(this)),
    )
      .map((id) => this.idTypeMapping.get(id))
      .filter(Boolean);
  }

  toString() {
    return `TypeScope(\nchildren=[\n${Array.from(this.astTypeMapping)
      .map(([key, ty]) => `${key} :: ${ty}`)
      .join(',\n')}\n],\n)`;
  }
}

function extractKey(
  keyable: Expression | MutName | Identifier | Path,
): string | number {
  return match(keyable)
    .with(P.instanceOf(Identifier), (ident) => ident.token.content)
    .with(P.instanceOf(Path), (path) => path.display)
    .with(P.instanceOf(MutName), (name) => extractKey(name.ident))
    .with(P.instanceOf(NameExpression), (expr) => extractKey(expr.path))
    .otherwise((expr) => expr.id.handle);
}
