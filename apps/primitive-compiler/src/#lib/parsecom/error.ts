export class ParseError<TInput> extends Error {
  readonly name = 'ParseError';

  constructor(
    message: string,
    readonly input: TInput,
    readonly code: ParseErrorCode,
    readonly cut: boolean,
  ) {
    super(message);
  }
}

export const ParseErrorCode = Object.freeze({
  Expectation: 'Expectation',
  Oneof: 'Oneof',
  Eof: 'Eof',
});
export type ParseErrorCode =
  (typeof ParseErrorCode)[keyof typeof ParseErrorCode];
