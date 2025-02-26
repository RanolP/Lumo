import { ParseError, ParseErrorCode } from './error';
import { InputSlice } from './types';

export class ArrayInput<TToken, TContext>
  implements InputSlice<TToken, TContext>
{
  _TNextThis: ArrayInput<TToken, TContext>;

  constructor(
    private array: TToken[],
    readonly context: TContext,
    private begin: number = 0,
  ) {}

  get intoInner(): TToken[] {
    return this.array.slice(this.begin);
  }

  split1(): [TToken, (typeof this)['_TNextThis']] {
    if (this.begin < this.array.length) {
      return [
        this.array[this.begin],
        new ArrayInput(this.array, this.context, this.begin + 1),
      ];
    } else {
      throw new ParseError('eof', this, ParseErrorCode.Expectation, false);
    }
  }

  updateContext<T>(
    update: (context: TContext) => [TContext, T],
  ): [T, (typeof this)['_TNextThis']] {
    const [nextContext, value] = update(this.context);
    return [value, new ArrayInput(this.array, nextContext, this.begin)];
  }
}
