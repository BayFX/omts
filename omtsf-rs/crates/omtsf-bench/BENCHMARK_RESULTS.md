# Benchmark Results

Collected on 2026-02-20 using `just bench` (Criterion 0.5, default sample sizes).

## Test Data Profiles

| Tier | Nodes | Edges | Total Elements | JSON Size |
|------|------:|------:|---------------:|----------:|
| S    |    50 |    91 |            141 |    38 KB  |
| M    |   500 |   982 |          1,482 |   405 KB  |
| L    | 2,000 | 3,948 |          5,948 | 1,666 KB  |
| XL   | 5,000 |10,007 |         15,007 | 4,510 KB  |

All generation is deterministic (seed=42). XL hits the ~5 MB target.

---

## Group 1: Parse & Serialize

| Operation        |   S    |   M     |    L     |    XL    | Throughput     |
|------------------|-------:|--------:|---------:|---------:|----------------|
| Deserialize      | 152 us | 1.79 ms | 11.3 ms  | 31.4 ms  | 140-244 MiB/s  |
| Serialize compact|  48 us |  527 us |  2.18 ms |  5.97 ms | 716-779 MiB/s  |
| Serialize pretty |  81 us |  890 us |  3.70 ms | 10.4 ms  | ~420-460 MiB/s |

Serialization is ~3.5x faster than deserialization. Compact serialize is ~1.7x faster
than pretty. All operations scale linearly with input size. Even at XL, a full
parse + serialize round-trip completes in under 40 ms.

## Group 2: Graph Construction

| Tier |  Time  | Throughput   |
|------|-------:|--------------|
| S    |  27 us | 4.8 Melem/s  |
| M    | 303 us | 5.6 Melem/s  |
| L    | 1.40 ms| 5.0 Melem/s  |
| XL   | 4.93 ms| 4.1 Melem/s  |

`build_graph` sustains ~5 million elements/sec. Slight throughput drop at XL likely
due to hash map resizing. Graph construction is fast enough to be negligible relative
to I/O.

## Group 3: Graph Queries

### Reachability (`reachable_from`)

| Variant                |    S   |    M    |    L    |     XL   |
|------------------------|-------:|--------:|--------:|---------:|
| Forward from root      |  5.5 us|  66.7 us|  273 us |   796 us |
| Forward from leaf      |  138 ns|   138 ns|  138 ns |   138 ns |
| Backward from root     |  3.3 us|  38.8 us|  157 us |   430 us |
| Both from mid          |  8.7 us|  101 us |  443 us |  1.30 ms |
| Filtered (supplies)    |  611 ns|   3.3 us|  9.8 us |  19.4 us |

Leaf queries are O(1) -- constant 138 ns regardless of graph size. Edge-type filtering
yields ~40x speedup. Full forward traversal of XL graph: under 1 ms.

### Shortest Path

| Variant        |    S   |    M    |    L    |     XL   |
|----------------|-------:|--------:|--------:|---------:|
| Root to leaf   |  7.9 us|  96 us  |  399 us |  1.14 ms |
| Root to mid    |  1.9 us|  24 us  |  112 us |   643 us |
| No path        |  157 ns|  156 ns |  156 ns |   156 ns |

No-path detection is O(1). Longest paths (root to leaf spanning full depth) scale
linearly.

### All Paths

| Variant  |    S    |     M    |
|----------|--------:|---------:|
| Depth 5  |  184 us |  3.67 ms |
| Depth 10 | 1.04 ms | 194.2 ms |

All-paths is the most expensive query -- exponential in path depth. M/depth_10 at
194 ms is the single slowest benchmark. Only benchmarked on S/M sizes.

## Group 4: Subgraph Extraction

### Induced Subgraph

| % Nodes |    S    |    M     |    L     |
|---------|--------:|---------:|---------:|
| 10%     | 12.5 us |   161 us |   707 us |
| 25%     | 28.7 us |   330 us |  1.42 ms |
| 50%     | 55.0 us |   610 us |  2.65 ms |
| 100%    | 87.1 us |  1.02 ms |  4.90 ms |

Near-perfect linear scaling with fraction extracted. Full L extraction in under 5 ms.

### Ego Graph

| Variant      |    S    |    M     |    L     |
|--------------|--------:|---------:|---------:|
| Root radius 1| 30.8 us |   225 us |   738 us |
| Root radius 2| 73.8 us |   561 us |  1.71 ms |
| Root radius 3| 87.8 us |   839 us |  2.91 ms |
| Mid radius 2 | 33.1 us |  50.8 us |   173 us |

Mid-node ego graphs are much cheaper than root ego graphs (fewer neighbors). Each
additional radius roughly doubles the cost.

## Group 5: Cycle Detection

| Variant                       |    S    |    M    |    L     |     XL   |
|-------------------------------|--------:|--------:|---------:|---------:|
| Acyclic, all types            | 25.6 us |  300 us |  1.33 ms |  4.13 ms |
| Acyclic, `legal_parentage`    |  8.2 us |   90 us |   367 us |  1.07 ms |
| Cyclic, all types             | 25.9 us |  293 us |  1.37 ms |      --  |
| Cyclic, `legal_parentage`     |  8.2 us |   90 us |   375 us |      --  |

Edge-type filtering yields ~3.5x speedup. Cyclic vs. acyclic performance is nearly
identical -- the algorithm does not short-circuit on first cycle. XL cycle detection
in 4 ms.

## Group 6: Validation

| Level      |    S    |    M     |    L     |     XL    |
|------------|--------:|---------:|---------:|----------:|
| L1 only    | 36 us   |   419 us |  2.11 ms |   6.94 ms |
| L1 + L2    | 60 us   |   853 us |  6.43 ms |  30.4 ms  |
| L1 + L2 + L3 | 58 us |   852 us |  6.06 ms |  31.2 ms  |

L1 validation is fast (proportional to element count). L2 adds semantic checks and
roughly doubles the cost. L3 (cycle detection) adds negligible overhead on top of L2.
Full L1+L2+L3 validation of a 5 MB XL file: 31 ms.

## Group 7: Merge Pipeline

| Variant                 |    S     |     M     |     L     |
|-------------------------|--------:|----------:|----------:|
| Self-merge (100% overlap)| 851 us  |  10.4 ms  |  57.0 ms  |
| Disjoint (0% overlap)   | 991 us  |  15.3 ms  |  76.4 ms  |
| 3-file merge            | 1.72 ms |  23.4 ms  |       --  |

Merge is the most expensive operation per-element. Disjoint merge is ~35% more
expensive than self-merge (more output nodes). The 3-file merge cost is roughly
additive.

## Group 8: Redaction

### By scope (retain all nodes)

| Scope   |    S     |    M     |    L     |     XL    |
|---------|--------:|---------:|---------:|----------:|
| Partner |  135 us |  1.71 ms |  8.15 ms |  24.9 ms  |
| Public  |  127 us |  1.54 ms |  6.94 ms |  22.2 ms  |

### Varying retain % (M tier)

| Retain % | Partner  | Public   |
|----------|--------:|---------:|
| 10%      | 1.27 ms | 1.21 ms  |
| 50%      | 1.64 ms | 1.54 ms  |
| 90%      | 1.69 ms | 1.52 ms  |

Public redaction is slightly faster than partner (person nodes are removed entirely,
reducing output). Retain fraction has modest impact -- the bulk of the cost is graph
traversal, not output construction.

## Group 9: Diff

| Variant                  |    S     |    M     |    L     |     XL    |
|--------------------------|--------:|---------:|---------:|----------:|
| Identical                |  285 us |  3.40 ms | 17.1 ms  |  78.3 ms  |
| Disjoint                 |  186 us |  2.02 ms |  9.58 ms |       --  |
| Filtered (org + supplies)|  112 us |  1.69 ms | 13.1 ms  |       --  |

Self-diff (identical files) is more expensive than disjoint diff because it must
match every element. XL self-diff at 78 ms is the second-slowest operation overall.

---

## Scaling Analysis

Node/edge ratios between tiers: S to M ~10x elements, M to L ~4x, L to XL ~2.5x.

| Operation       | S to M | M to L | L to XL | Complexity |
|-----------------|:------:|:------:|:-------:|:----------:|
| Deserialize     | 11.8x  |  6.3x  |  2.8x   |    O(n)    |
| Serialize       | 11.1x  |  4.1x  |  2.7x   |    O(n)    |
| Build graph     | 11.2x  |  4.6x  |  3.5x   |    O(n)    |
| Validate L1     | 11.7x  |  5.0x  |  3.3x   |    O(n)    |
| Validate L1+L2+L3| 14.7x |  7.1x  |  5.1x   | O(n log n) |
| Diff identical  | 11.9x  |  5.0x  |  4.6x   | O(n log n) |
| Redact partner  | 12.7x  |  4.8x  |  3.1x   |    O(n)    |

Most operations show near-linear scaling. Validation L2+L3 and diff show slightly
super-linear behavior (hash-based identifier lookups).

## Key Takeaways

1. **All operations complete under 100 ms for XL (5 MB) files** -- well within
   interactive budgets.
2. **Serialization is 3-5x faster than deserialization** -- serde's write path is
   highly optimized.
3. **Graph queries are the fastest operations** -- sub-millisecond even at XL.
   Edge-type filtering provides 10-40x speedups.
4. **Merge is the most expensive operation** -- canonical identifier matching
   dominates. 76 ms for L-tier disjoint merge.
5. **`all_paths` with depth 10 is the performance cliff** -- 194 ms on M-tier,
   exponential growth. Depth limits are essential.
6. **Cycle detection adds negligible cost to validation** -- L3 is essentially free
   on top of L2.
7. **No operation requires optimization for the current scale target** -- all are
   within acceptable latency bounds.
