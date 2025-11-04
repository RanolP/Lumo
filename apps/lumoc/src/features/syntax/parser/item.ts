import { handsum, Handsum } from 'handsum';
import { RefinedTypeV } from '../../type';
import { Computation } from '../../ast/computation';

interface TItem {
  LetType(name: string, ty: RefinedTypeV): Item;
  LetComputation(name: string, comp: Computation): Item;
}
interface IItem {
  display(this: Item): string;
}
export type Item = Handsum<TItem, IItem>;
export const Item = handsum<TItem, IItem>({
  display() {
    return this.match({
      LetType(name, ty) {
        return `let-type ${name} = ${ty.display()}`;
      },
      LetComputation(name, comp) {
        return `let-computation ${name} = ${comp.display()}`;
      },
    });
  },
});
