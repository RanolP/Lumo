import type { Simplify } from 'type-fest';

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

    get leftoverTokens() {
      return items.slice(index);
    },
  } satisfies Input<{
    next(): TItem | null;
  }> & {
    leftoverTokens: TItem[];
  };
};
export type ArrayInput<TItem> = ReturnType<typeof createArrayInput<TItem>>;

export const createContextfulInput =
  <TContext>(initialContext: TContext) =>
  <TInstructions extends {}, TInput extends Input<TInstructions>>(
    input: TInput,
  ): ContextfulInput<TContext, TInput> => {
    let context = initialContext;

    return new Proxy<ContextfulInput<TContext, TInput>>(input as any, {
      get(target, prop) {
        if (prop === 'checkpoint') {
          return () => {
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
          };
        }
        return target[prop as keyof ContextfulInput<TContext, TInput>];
      },
    });
  };
export type ContextfulInput<TContext, TInput extends Input<{}>> = Simplify<
  Omit<TInput, 'checkpoint'>
> &
  Input<
    TInput extends Input<infer TInstructions>
      ? TInstructions & { context: TContext }
      : never
  >;
