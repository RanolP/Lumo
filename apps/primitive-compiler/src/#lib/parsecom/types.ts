export type ParseFn<TInput, TOutput> = (input: TInput) => [TInput, TOutput];

export interface Parser<TInput, TOutput> extends ParseFn<TInput, TOutput> {
  map<U>(f: (output: TOutput) => U): Parser<TInput, U>;

  flatMap<U>(p: (output: TOutput) => Parser<TInput, U>): Parser<TInput, U>;
}

export interface InputSlice<TToken, TContext = {}> {
  _TNextThis: InputSlice<TToken, TContext>;

  readonly context: TContext;

  split1(): [TToken, (typeof this)['_TNextThis']];

  updateContext<T>(
    update: (context: TContext) => [TContext, T],
  ): [T, (typeof this)['_TNextThis']];
}
