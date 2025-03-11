export type TypeGraph = Map<string, string[]>;

export function findAncestors(
  graph: TypeGraph,
  node: string,
  visited: Set<string>,
): Set<string> {
  if (visited.has(node)) return new Set();
  visited.add(node);

  const ancestors = new Set(graph.get(node) ?? []);
  for (const parent of ancestors) {
    for (const ancestor of findAncestors(graph, parent, visited)) {
      ancestors.add(ancestor);
    }
  }

  return ancestors;
}

export function findLowestCommonAncestor(
  graph: TypeGraph,
  a: string,
  b: string,
): Set<string> {
  const ancestorsA = findAncestors(graph, a, new Set());
  const ancestorsB = findAncestors(graph, b, new Set());

  const commonAncestors = new Set(
    [...ancestorsA].filter((x) => ancestorsB.has(x)),
  );

  // Find the most specific one
  const lcaCandidates = new Set(commonAncestors);
  for (const ancestor of commonAncestors) {
    for (const other of commonAncestors) {
      if (ancestor !== other && graph.get(other)?.includes(ancestor)) {
        lcaCandidates.delete(ancestor);
      }
    }
  }

  return lcaCandidates;
}
