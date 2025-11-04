export interface Input<TInstructions extends {} = {}> {
  checkpoint(): [instructions: TInstructions, apply: () => void];
}

export const createArrayInput = <TItem>(items: TItem[], begin: number = 0) => {
  let index = begin;

  return {
    checkpoint() {
      let newIndex = index;
      return [
        {
          next() {
            return items[newIndex++] ?? null;
          },
        },
        () => {
          index = newIndex;
        },
      ] as const;
    },
  } satisfies Input<{
    next(): TItem | null;
  }>;
};
export type ArrayInput<TItem> = ReturnType<typeof createArrayInput<TItem>>;

export const createContextfulInput =
  <TContext>(initialContext: TContext) =>
  <TInstructions extends {}>(
    input: Input<TInstructions>,
  ): ContextfulInput<TContext, Input<TInstructions>> => {
    let context = initialContext;

    return {
      checkpoint() {
        const [inst, apply] = input.checkpoint();
        let newContext = context;

        return [
          Object.assign(inst, {
            get context() {
              return newContext;
            },
            set context(value: TContext) {
              newContext = value;
            },
          }),
          () => {
            apply();
            context = newContext;
          },
        ] as const;
      },
    } satisfies Input<
      TInstructions & {
        context: TContext;
      }
    >;
  };
export type ContextfulInput<TContext, TInput extends Input<{}>> = Input<
  TInput extends Input<infer TInstructions>
    ? TInstructions & { context: TContext }
    : never
>;
