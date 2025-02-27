import { ParseError, ParseErrorCode } from './error';
import { InputSlice, ParseFn, Parser } from './types';
import { UnknownArray } from 'type-fest';

interface Config<TToken, TContext> {
  dropTokenIf?: (ctx: TContext, token: TToken) => boolean;
}
export function parsecom<TToken, TContext extends {} = {}>({
  dropTokenIf,
}: Config<TToken, TContext> = {}) {
  type Input = InputSlice<TToken, TContext>;

  const makeParser = <TOutput>(
    fn: ParseFn<Input, TOutput>,
  ): Parser<Input, TOutput> => {
    const self = Object.assign(fn, {
      map: <U>(f: (output: TOutput) => U) => map(self, f),
      flatMap: <U>(f: (output: TOutput) => Parser<Input, U>) =>
        flatMap(self, f),
    });
    return self;
  };

  const map = <TOutput, U>(
    parser: Parser<Input, TOutput>,
    f: (output: TOutput) => U,
  ): Parser<Input, U> =>
    makeParser((i) => {
      let output: TOutput, flatMapOutput: U;
      [i, output] = parser(i);
      flatMapOutput = f(output);

      return [i, flatMapOutput];
    });

  const flatMap = <TOutput, U>(
    parser: Parser<Input, TOutput>,
    f: (output: TOutput) => Parser<Input, U>,
  ): Parser<Input, U> =>
    makeParser((i) => {
      let output: TOutput, flatMapOutput: U;
      [i, output] = parser(i);
      [i, flatMapOutput] = f(output)(i);

      return [i, flatMapOutput];
    });

  const seq = <TSeq extends UnknownArray>(
    ...parsers: SeqParser<TSeq>
  ): Parser<Input, TSeq> =>
    makeParser((i: Input) => {
      const seqResult: unknown[] = [];
      for (const parser of parsers) {
        let result: unknown;
        [i, result] = parser(i);
        seqResult.push(result);
      }
      return [i, seqResult as unknown as TSeq];
    });
  const repeat0 = <TOutput>(
    parser: Parser<Input, TOutput>,
  ): Parser<Input, TOutput[]> =>
    makeParser((i) => {
      const repeat0Result: TOutput[] = [];
      while (true) {
        let result: TOutput | null;
        [i, result] = opt(parser)(i);
        if (result == null) break;
        repeat0Result.push(result);
      }
      return [i, repeat0Result];
    });

  const ctx = <TOutput>(
    update: (ctx: TContext) => [TContext, TOutput],
  ): Parser<Input, TOutput> =>
    makeParser((i) => {
      let value: TOutput;
      [value, i] = i.updateContext(update);
      return [i, value];
    });

  const rule = <TOutput>(
    parser: () => Parser<Input, TOutput>,
  ): Parser<Input, TOutput> => makeParser((i) => parser()(i));

  const takeIf = (
    filter: (token: TToken) => boolean,
    expectation: (token: TToken) => string,
  ): Parser<Input, TToken> =>
    makeParser((i) => {
      let token: TToken;
      while (true) {
        [token, i] = i.split1();
        if (
          typeof dropTokenIf === 'function' &&
          dropTokenIf(i.context, token)
        ) {
          continue;
        }
        if (!filter(token)) {
          throw new ParseError(
            `Expected ${expectation(token)}`,
            i,
            ParseErrorCode.Expectation,
            false,
          );
        }
        return [i, token];
      }
    });

  const opt = <TOutput>(
    parser: Parser<Input, TOutput>,
  ): Parser<Input, TOutput | null> =>
    makeParser((i: Input) => {
      try {
        return parser(i);
      } catch (e) {
        if (e instanceof ParseError && e.cut) {
          throw e;
        }
        return [i, null];
      }
    });

  type SeqParser<Array extends UnknownArray> = {
    [K in keyof Array]: Parser<Input, Array[K]>;
  };

  return {
    rule,
    make: {
      tag:
        <T>(
          pick: (token: TToken) => T,
          expectation: (token: TToken) => string,
        ) =>
        (tag: T): Parser<Input, TToken> =>
          takeIf((token) => pick(token) === tag, expectation),
    },
    cut: <TOutput>(parser: Parser<Input, TOutput>): Parser<Input, TOutput> =>
      makeParser((i) => {
        try {
          return parser(i);
        } catch (e) {
          if (e instanceof ParseError) {
            throw new ParseError(e.message, e.input, e.code, true);
          }
          throw e;
        }
      }),
    ctx,
    withCtxMod:
      <K extends keyof TContext>(key: K, value: TContext[K]) =>
      <TOutput>(p: Parser<Input, TOutput>) =>
        rule<TOutput>(() =>
          seq(
            ctx(({ [key]: old, ...rest }) => [
              { [key]: value, ...rest } as TContext,
              old,
            ]),
            p,
          ).flatMap(([old, ret]) =>
            ctx(({ [key]: _, ...rest }) => [
              { [key]: old, ...rest } as TContext,
              ret,
            ]),
          ),
        ),

    takeIf,
    opt,

    repeat0,
    separatedList1: <TItem>(
      item: Parser<Input, TItem>,
      sep: Parser<Input, unknown>,
    ): Parser<Input, TItem[]> =>
      rule(() => seq(item, repeat0(seq(sep, item)))).map(([first, list]) => [
        first,
        ...list.map(([_sep, it]) => it),
      ]),

    seq,
    oneof: <TSeq extends UnknownArray>(
      expectations: string,
      ...parsers: SeqParser<TSeq>
    ): Parser<Input, TSeq[number]> =>
      makeParser((i) => {
        for (const parser of parsers) {
          let result: unknown;
          [i, result] = opt(parser)(i);
          if (result != null) {
            return [i, result];
          }
        }
        throw new ParseError(
          `Expected ${expectations}`,
          i,
          ParseErrorCode.Oneof,
          false,
        );
      }),

    failure: (message: string) =>
      makeParser<never>((i) => {
        throw new ParseError(message, i, ParseErrorCode.Failure, false);
      }),

    __Parser: <TOutput>() => null as unknown as Parser<Input, TOutput>,
  };
}

export { ArrayInput } from './input';
export { Parser, InputSlice } from './types';
