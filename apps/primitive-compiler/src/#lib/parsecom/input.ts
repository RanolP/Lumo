import { ParseError, ParseErrorCode } from './error.js';
import { InputSlice } from './types.js';

export class ArrayInput<TToken, TContext>
  implements InputSlice<TToken, TContext>
{
  _TNextThis: ArrayInput<TToken, TContext> = this;

  constructor(
    private array: TToken[],
    readonly context: TContext,
    readonly position: number = 0,
  ) {}

  get intoInner(): TToken[] {
    return this.array.slice(this.position);
  }

  split1(): [TToken, (typeof this)['_TNextThis']] {
    if (this.position < this.array.length) {
      return [
        this.array[this.position],
        new ArrayInput(this.array, this.context, this.position + 1),
      ];
    } else {
      throw new ParseError('eof', this, ParseErrorCode.Expectation, false);
    }
  }

  updateContext<T>(
    update: (context: TContext) => [TContext, T],
  ): [T, (typeof this)['_TNextThis']] {
    const [nextContext, value] = update(this.context);
    return [value, new ArrayInput(this.array, nextContext, this.position)];
  }
}
