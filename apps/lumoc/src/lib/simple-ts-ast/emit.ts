import type { Key, TsExpr, TsType } from '.';

export function emitType(type: TsType): string {
  return type.match({
    Variable(name) {
      return name;
    },
    Lambda(param, typeParams, body) {
      return `${
        typeParams.length > 0 ? `<${typeParams.join(', ')}>` : ''
      }(${param
        .map(({ name, type }) => `${name}${type ? `: ${emitType(type)}` : ''}`)
        .join(', ')}) => ${emitType(body)}`;
    },
    UntaggedUnion(types) {
      return types.length > 0
        ? `(${types.map((t) => `(${emitType(t)})`).join(' | ')})`
        : 'never';
    },
    Object(entries) {
      return `{${entries
        .map(({ name, type }) => `${objectkey(name)}: (${emitType(type)})`)
        .join(', ')}}`;
    },
    StringLiteral(value) {
      return `"${value}"`;
    },
    TypeApplication(body, typeParams) {
      return `${emitType(body)}${
        typeParams.length > 0
          ? `<${typeParams.map((t) => emitType(t)).join(', ')}>`
          : ''
      }`;
    },
  });
}

export function emitExpr(expr: TsExpr): string {
  return expr.match({
    Variable(name) {
      return name;
    },
    Apply(fn, params) {
      return `${emitExpr(fn)}(${params.map((p) => emitExpr(p)).join(', ')})`;
    },
    Lambda(params, typeParams, ret, body) {
      return `(${
        typeParams.length > 0 ? `<${typeParams.join(', ')}>` : ''
      }(${params
        .map(({ name, type }) => `${name}${type ? `: ${emitType(type)}` : ''}`)
        .join(', ')})${ret ? `: ${emitType(ret)}` : ''} => ${emitExpr(body)})`;
    },
    FieldAccess(object, field) {
      return `${emitExpr(object)}${fieldAccess(field)}`;
    },
    Object(entries) {
      return `({${entries
        .map(({ name, value }) => `${objectkey(name)}: ${emitExpr(value)}`)
        .join(', ')}})`;
    },
    StringLiteral(value) {
      return `"${value}"`;
    },
    Satisfies(expr, type) {
      return `(${emitExpr(expr)} satisfies ${emitType(type)})`;
    },
    Ternary(condition, then, otherwise) {
      return `(${emitExpr(condition)} ? ${emitExpr(then)} : ${emitExpr(
        otherwise,
      )})`;
    },
    Never() {
      return 'std.Never()';
    },
    Equals(left, right) {
      return `(${emitExpr(left)} === ${emitExpr(right)})`;
    },
    TypeApplication(body, typeParams) {
      return `${emitExpr(body)}${
        typeParams.length > 0
          ? `<${typeParams.map((t) => emitType(t)).join(', ')}>`
          : ''
      }`;
    },
  });
}

function objectkey(key: Key): string {
  switch (key.tag) {
    case 'string':
      if (/\p{ID_Start}\p{ID_Continue}*/u.test(key.value)) {
        return key.value;
      }
      return `[${JSON.stringify(key.value)}]`;
    case 'symbol':
      switch (key.value) {
        case 'Lumo/tag':
          return '[STD_TAG]';
        default:
          throw new Error('Invalid symbol key: ' + JSON.stringify(key));
      }
  }
}

function fieldAccess(key: Key): string {
  switch (key.tag) {
    case 'string':
      if (/\p{ID_Start}\p{ID_Continue}*/u.test(key.value)) {
        return `.${key.value}`;
      }
      return `[${key.value}]`;
    case 'symbol':
      switch (key.value) {
        case 'Lumo/tag':
          return '[STD_TAG]';
        default:
          throw new Error('Invalid symbol key: ' + JSON.stringify(key));
      }
  }
}
