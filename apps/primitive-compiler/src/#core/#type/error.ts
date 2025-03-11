import { Span } from '@/#syntax/index.js';

export class TypingError extends Error {
  readonly name = 'TypingError';

  constructor(message: string, readonly node: { span: Span } | null) {
    super(message);
  }
}
