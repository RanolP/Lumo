type Cx = 'ty' | 'comput' | 'val';

let state = {
  ty: 0,
  comput: 0,
  val: 0,
};

export function freshName(cx: Cx): string {
  return toString(cx, state[cx]++);
}

const tyMap = 'αβγδεζηθικλμνξοπρστυφχψω';
const computMap = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ';
const valMap = 'abcdefghijklmnopqrstuvwxyz';

const superscripts = ['', ...'²³⁴⁵⁶⁷⁸⁹'];

function toString(cx: Cx, index: number): string {
  switch (cx) {
    case 'ty':
      return (
        tyMap[index % tyMap.length]! +
        superscripts[Math.floor(index / tyMap.length)]!
      );
    case 'comput':
      return (
        computMap[index % computMap.length]! +
        superscripts[Math.floor(index / computMap.length)]!
      );
    case 'val':
      return (
        valMap[index % valMap.length]! +
        superscripts[Math.floor(index / valMap.length)]!
      );
  }
}
