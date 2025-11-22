export class Hasher {
  #result: number = 0;

  with(value: string | number | { hashCode: () => number }): this {
    if (typeof value === 'string') {
      let hash = 0;
      for (let i = 0; i < value.length; i++) {
        hash = (hash << 5) - hash + value.charCodeAt(i);
      }
      this.#result = this.#result * 31 + hash;
    } else if (typeof value === 'number') {
      this.#result = this.#result * 31 + value;
    } else {
      this.#result = this.#result * 31 + value.hashCode();
    }
    return this;
  }

  result(): number {
    return this.#result;
  }
}
