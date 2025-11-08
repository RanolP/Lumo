let state = 0;
export function freshName(): string {
  return `#FreshName${state++}`;
}
