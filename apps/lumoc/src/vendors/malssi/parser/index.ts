import { Input } from './input';
import { Simplify } from 'type-fest';
import { ParseError } from './errors';

function implParser<
  TInput extends Input,
  TOutput,
  const CaptureName extends string | undefined,
>({
  run,
  label,
  captureName,
}: {
  run: (input: TInput) => TOutput;
  label: string | undefined;
  captureName: CaptureName;
}): Parser<TInput, TOutput, CaptureName> {
  const self = {
    '~capture': captureName,
    run: run,
    map<UOutput>(
      f: (output: TOutput) => UOutput,
    ): Parser<TInput, UOutput, CaptureName> {
      return implParser({
        run: (input) => f(run(input)),
        label,
        captureName,
      });
    },
    capture<TName extends string>(name: TName): Parser<TInput, TOutput, TName> {
      return implParser({ run, label, captureName: name });
    },
    void(): Parser<TInput, void, CaptureName> {
      return self.map(() => undefined);
    },
    labeled(label: string): Parser<TInput, TOutput, CaptureName> {
      return implParser({
        run: (input) => {
          return run(input);
        },
        label,
        captureName,
      });
    },
    opt(): Parser<TInput, TOutput | undefined, CaptureName> {
      return implParser({
        run: (input) => {
          try {
            return run(input);
          } catch (error) {
            if (error instanceof ParseError) return undefined;

            throw error;
          }
        },
        label,
        captureName,
      });
    },
    or<UOutput>(
      other: Parser<TInput, UOutput, unknown>,
    ): Parser<TInput, TOutput | UOutput, CaptureName> {
      return implParser({
        run: (input) => {
          try {
            return run(input);
          } catch (error) {
            if (error instanceof ParseError) return other.run(input);

            throw error;
          }
        },
        label,
        captureName,
      });
    },
    repeat(
      min: number = 0,
      max: number = Infinity,
    ): Parser<TInput, TOutput[], CaptureName> {
      return implParser({
        run: (input) => {
          const results = [];
          let count = 0;
          while (count < max) {
            try {
              const result = run(input);
              results.push(result);
            } catch {
              break;
            }
          }
          if (min !== undefined && results.length < min) {
            throw new ParseError(input, `expected at least ${min} items`);
          }
          return results;
        },
        label,
        captureName,
      });
    },
    sepBy(
      sep: Parser<TInput, any, void>,
      min: number = 0,
      max?: number,
    ): Parser<TInput, TOutput[], CaptureName> {
      return implParser({
        run: (input) => {
          const seq = _seq<TInput>();
          const [, rollback] = input.checkpoint();
          try {
            const head = self.run(input);
            const tail = seq<
              [Parser<TInput, any, void>, Parser<TInput, TOutput, 'item'>]
            >(sep, self.capture('item'))
              .repeat(Math.max(min - 1, 0), max)
              .map((res) =>
                res.map((i) => (i as unknown as { item: TOutput }).item),
              )
              .run(input);
            return [head, ...tail];
          } catch (e) {
            rollback();
            if (min === 0 && e instanceof ParseError) return [];
            throw e;
          }
        },
        label,
        captureName,
      });
    },
  };
  return self as Parser<TInput, TOutput, CaptureName>;
}

export const malssi = <TInput extends Input>() =>
  Object.assign(
    <TOutput>(
      f: () => Parser<TInput, TOutput, void>,
    ): Parser<TInput, TOutput, void> => {
      return implParser({
        run: (input) => {
          const result = f().run(input);
          return result;
        },
        label: undefined,
        captureName: undefined,
      });
    },
    {
      raw: <TOutput>(
        f: (i: ReturnType<TInput['checkpoint']>[0]) => TOutput,
      ): Parser<TInput, TOutput, void> =>
        implParser({
          run: (i) => {
            const [inst, apply] = i.checkpoint();
            const result = f(inst);
            apply();
            return result;
          },
          label: undefined,
          captureName: undefined,
        }),
      seq: _seq<TInput>(),
    },
  );

const _seq =
  <TInput extends Input>() =>
  <
    const TParsers extends [
      Parser<TInput, any, unknown>,
      ...Parser<TInput, any, unknown>[],
    ],
  >(
    ...parsers: TParsers
  ) =>
    implParser({
      run: (input: TInput): Simplify<AggregateSeq<TInput, TParsers>> => {
        const [, rollback] = input.checkpoint();
        try {
          let aggregated: Record<string, unknown> = {};

          for (const parser of parsers) {
            const result = parser.run(input);
            if (typeof parser['~capture'] === 'string') {
              aggregated[parser['~capture']] = result;
            }
          }

          return aggregated as any;
        } catch (e) {
          rollback();
          throw e;
        }
      },
      label: undefined,
      captureName: undefined,
    });

export type Parser<TInput extends Input, TOutput, CaptureName> = Simplify<
  {
    '~label'?: string;
    '~capture': CaptureName;

    run(input: TInput): TOutput;
    map<UOutput>(
      f: (output: TOutput) => UOutput,
    ): Parser<TInput, UOutput, CaptureName>;
    opt(): Parser<TInput, TOutput | undefined, CaptureName>;
    or<UOutput>(
      other: Parser<TInput, UOutput, unknown>,
    ): Parser<TInput, TOutput | UOutput, CaptureName>;
    repeat(min?: number, max?: number): Parser<TInput, TOutput[], CaptureName>;
    sepBy(
      parser: Parser<TInput, any, void>,
      min?: number,
      max?: number,
    ): Parser<TInput, TOutput[], CaptureName>;
    labeled(label: string): Parser<TInput, TOutput, CaptureName>;
  } & ([CaptureName] extends [void]
    ? {
        capture<TName extends string>(
          name: TName,
        ): Parser<TInput, TOutput, TName>;
      }
    : {})
> &
  ([TOutput] extends [void]
    ? {}
    : {
        void(): Parser<TInput, void, CaptureName>;
      });

type AggregateSeq<TInput extends Input, T> = T extends [infer H, ...infer T]
  ? (H extends Parser<TInput, infer O, infer N extends string>
      ? Record<N, O>
      : {}) &
      AggregateSeq<TInput, T>
  : {};
