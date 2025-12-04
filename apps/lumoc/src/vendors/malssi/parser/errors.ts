export class ParseError<TInput> extends Error {
  input: TInput;

  constructor(input: TInput, message: string) {
    super(message);
    this.name = 'ParseError';
    this.input = input;
  }
}

export class ExpectedError<TInput> extends ParseError<TInput> {
  constructor(input: TInput, expected: string, actual: string) {
    super(input, `expected ${expected}, got ${actual}`);
    this.name = 'ExpectedError';
  }
}

export class OneOfError<TInput> extends ParseError<TInput> {
  constructor(input: TInput, expected: string[]) {
    super(input, `expected one of ${expected.join(', ')}`);
    this.name = 'OneOfError';
  }
}

export class ExpectedAtLeastError<TInput> extends ParseError<TInput> {
  constructor(input: TInput, expected: number) {
    super(input, `expected at least ${expected} items`);
    this.name = 'ExpectedAtLeastError';
  }
}
