const Span$ = Symbol('Span');

/**
 * Span is a range with inclusive begin and exclusive end.
 * And it's maybe EOF if `begin === -1n`
 */
export type Span = {
  __tag$: typeof Span$;
  toString(): string;
  isEof: boolean;

  begin: bigint;
  end: bigint;
};
function make(
  data: Omit<Span, '__tag$' | typeof Symbol.toStringTag | 'isEof'>,
): Span {
  return {
    ...data,
    toString() {
      return `Span[${data.begin}..${data.end}]`;
    },
    isEof: data.begin === -1n,
    __tag$: Span$,
  };
}
export const Span = { make };
