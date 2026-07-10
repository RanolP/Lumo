# Cost model: adopt a widely-used model

For e-graph extraction, do not invent a cost model — **adopt a widely
adopted one**, selected by a research pass. Candidates to survey: egg /
egglog per-constructor costs with minimal-total-cost extraction (`:cost`
annotations), and the extraction-gym line of work (greedy vs
DAG-aware/ILP extractors). The research pass is a pending action item.

**Resolved by decision 31**: per-constructor constant costs (default 1)
with egglog's built-in min-tree-cost extraction.
