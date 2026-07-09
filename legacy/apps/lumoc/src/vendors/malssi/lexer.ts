export function lexer<TTokenSet>() {
  const rules: Array<{
    pattern: RegExp;
    transform: (groups: Record<string, string>) => unknown;
  }> = [];
  return {
    lex(src: string) {
      const sum = rules
        .map((v, i) => [v, i] as const)
        .reduce(
          (acc, [{ pattern }, key]) =>
            [acc, `(?<MalssiLexer__${key}>${pattern.source})`]
              .filter(Boolean)
              .join('|'),
          '',
        );
      const transformers: Record<
        string,
        (groups: Record<string, string>) => unknown
      > = rules
        .map((v, i) => [v, i] as const)
        .reduce(
          (acc, [{ transform }, key]) => ({ ...acc, [key]: transform }),
          {},
        );

      const fullRegexp = new RegExp(`^(${sum})(?<MalssiLexer__rest>.*)$`, 'su');

      const result = [];
      let match: Record<string, string> | undefined;
      while ((match = fullRegexp.exec(src)?.groups)) {
        const { MalssiLexer__rest: rest, ...groups } = match;
        const entries = Object.entries(groups)
          .filter(([key]) => key.startsWith('MalssiLexer__'))
          .sort(([_a, a], [_b, b]) => (!b ? -1 : !a ? 1 : a.length - b.length));
        if (entries.length === 0) break;

        const key = entries[0]![0].slice('MalssiLexer__'.length);
        result.push(transformers[key]!(groups));
        src = rest ?? '';
      }
      return result as any;
    },
    rule(
      pattern: RegExp,
      transform: (groups: Record<string, string>) => unknown,
    ) {
      rules.push({ pattern, transform });
      return this;
    },
    ruleset(newRules: [RegExp, (groups: Record<string, string>) => unknown][]) {
      rules.push(
        ...newRules.map(([pattern, transform]) => ({ pattern, transform })),
      );
      return this;
    },
  } as Lexer<TTokenSet>;
}

export interface Lexer<TTokenSet = never> {
  lex(src: string): TTokenSet[];

  rule: (
    pattern: RegExp,
    transform: (groups: Record<string, string>) => TTokenSet,
  ) => Lexer<TTokenSet>;

  ruleset: <
    TRules extends [RegExp, (groups: Record<string, string>) => TTokenSet][],
  >(
    rules: TRules,
  ) => Lexer<TTokenSet>;
}
