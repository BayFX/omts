/// Subgraph extraction algorithms: induced subgraph, ego-graph, and selector-based extraction.
///
/// Implements Section 5 of the graph-engine technical specification.
///
/// # Induced Subgraph
///
/// [`induced_subgraph`] accepts a set of node IDs and returns an [`OmtsFile`]
/// containing exactly those nodes and every edge whose source *and* target are
/// both in the set.
///
/// # Ego-Graph
///
/// [`ego_graph`] wraps a bounded BFS around [`induced_subgraph`]: it first
/// collects all nodes within `radius` hops of the `center` node (inclusive of
/// the center), then extracts the induced subgraph of that neighbourhood.
///
/// # Selector-Based Extraction
///
/// [`selector_match`] scans all nodes and edges for property-predicate matches
/// without assembling a subgraph. [`selector_subgraph`] performs the full
/// pipeline: seed scan → seed edge resolution → BFS expansion → induced
/// subgraph assembly.
///
/// # Output Validity
///
/// All extraction functions return a valid [`OmtsFile`] with the original
/// header fields preserved.  The `reporting_entity` header field is retained
/// only if the referenced node is present in the subgraph; otherwise it is
/// set to `None`.
use std::collections::{HashSet, VecDeque};

use petgraph::stable_graph::NodeIndex;
use petgraph::visit::EdgeRef;

use crate::file::OmtsFile;
use crate::graph::OmtsGraph;
use crate::graph::queries::{Direction, QueryError};
use crate::graph::selectors::SelectorSet;

#[cfg(test)]
mod tests;

/// Extracts the induced subgraph for the given set of node IDs.
///
/// The induced subgraph contains exactly the specified nodes and every edge
/// in the original graph whose source *and* target are both in the node set.
/// Graph-local IDs are preserved so edge `source`/`target` references remain
/// correct in the returned file.
///
/// # Parameters
///
/// - `graph` — the graph to query.
/// - `file` — the source [`OmtsFile`] used to build `graph`; provides the full
///   node and edge data, and the header fields to carry forward.
/// - `node_ids` — graph-local IDs of the nodes to include.
///
/// # Output
///
/// Returns a valid [`OmtsFile`] with:
/// - The original header fields (`omtsf_version`, `snapshot_date`,
///   `file_salt`, `disclosure_scope`, `previous_snapshot_ref`,
///   `snapshot_sequence`, `extra`) preserved as-is.
/// - `reporting_entity` retained only if the referenced node is present in
///   the subgraph; otherwise `None`.
/// - `nodes` and `edges` filtered to the induced subgraph.
///
/// # Errors
///
/// Returns [`QueryError::NodeNotFound`] if any ID in `node_ids` does not
/// exist in the graph.
pub fn induced_subgraph(
    graph: &OmtsGraph,
    file: &OmtsFile,
    node_ids: &[&str],
) -> Result<OmtsFile, QueryError> {
    let mut index_set: HashSet<NodeIndex> = HashSet::with_capacity(node_ids.len());
    for &id in node_ids {
        let idx = *graph
            .node_index(id)
            .ok_or_else(|| QueryError::NodeNotFound(id.to_owned()))?;
        index_set.insert(idx);
    }

    assemble_subgraph(graph, file, &index_set)
}

/// Extracts the ego-graph: the induced subgraph of all nodes within `radius`
/// hops of `center`.
///
/// Algorithm:
/// 1. Run a bounded BFS from `center`, collecting every node reachable within
///    `radius` hops (inclusive of the center itself).
/// 2. Extract the induced subgraph of the collected node set.
///
/// This is equivalent to the `radius`-neighbourhood of `center` in `graph`.
///
/// # Parameters
///
/// - `graph` — the graph to query.
/// - `file` — the source [`OmtsFile`] used to build `graph`.
/// - `center` — graph-local ID of the ego node.
/// - `radius` — maximum number of hops from `center` (0 returns only the
///   center node; 1 returns the center plus its direct neighbours; etc.).
/// - `direction` — which edges to follow when expanding the neighbourhood.
///
/// # Errors
///
/// Returns [`QueryError::NodeNotFound`] if `center` does not exist in the
/// graph.
pub fn ego_graph(
    graph: &OmtsGraph,
    file: &OmtsFile,
    center: &str,
    radius: usize,
    direction: Direction,
) -> Result<OmtsFile, QueryError> {
    let center_idx = *graph
        .node_index(center)
        .ok_or_else(|| QueryError::NodeNotFound(center.to_owned()))?;

    let mut visited: HashSet<NodeIndex> = HashSet::new();
    let mut queue: VecDeque<(NodeIndex, usize)> = VecDeque::new();

    visited.insert(center_idx);
    queue.push_back((center_idx, 0));

    while let Some((current, hops)) = queue.pop_front() {
        if hops >= radius {
            continue;
        }

        let g = graph.graph();
        let next_hops = hops + 1;

        match direction {
            Direction::Forward => {
                for edge_ref in g.edges(current) {
                    let neighbour = edge_ref.target();
                    if !visited.contains(&neighbour) {
                        visited.insert(neighbour);
                        queue.push_back((neighbour, next_hops));
                    }
                }
            }
            Direction::Backward => {
                for edge_ref in g.edges_directed(current, petgraph::Direction::Incoming) {
                    let neighbour = edge_ref.source();
                    if !visited.contains(&neighbour) {
                        visited.insert(neighbour);
                        queue.push_back((neighbour, next_hops));
                    }
                }
            }
            Direction::Both => {
                for edge_ref in g.edges(current) {
                    let neighbour = edge_ref.target();
                    if !visited.contains(&neighbour) {
                        visited.insert(neighbour);
                        queue.push_back((neighbour, next_hops));
                    }
                }
                for edge_ref in g.edges_directed(current, petgraph::Direction::Incoming) {
                    let neighbour = edge_ref.source();
                    if !visited.contains(&neighbour) {
                        visited.insert(neighbour);
                        queue.push_back((neighbour, next_hops));
                    }
                }
            }
        }
    }

    assemble_subgraph(graph, file, &visited)
}

/// Result of a [`selector_match`] scan.
///
/// Contains the indices into the originating `OmtsFile`'s `nodes` and `edges`
/// vectors for all elements that matched the given selectors.
#[derive(Debug, Default)]
pub struct SelectorMatchResult {
    /// Indices into `file.nodes` for matching nodes.
    pub node_indices: Vec<usize>,
    /// Indices into `file.edges` for matching edges.
    pub edge_indices: Vec<usize>,
}

/// Returns the indices of all nodes and edges in `file` that match `selectors`.
///
/// Performs a single linear scan of `file.nodes` and `file.edges` — O((N + E) * S)
/// where S is the total number of selector values. Does **not** perform neighbor
/// expansion or assemble a subgraph file.
///
/// Intended for the `omtsf query` command, which displays matches without
/// producing a new `.omts` file.
///
/// When `selectors` is empty, every node and edge is returned (universal match).
/// Otherwise, nodes are evaluated only when `selectors` contains at least one
/// node-applicable selector (see [`SelectorSet::has_node_selectors`]), and edges
/// are evaluated only when `selectors` contains at least one edge-applicable selector
/// (see [`SelectorSet::has_edge_selectors`]).  This ensures that an edge-only
/// `SelectorSet` returns no nodes, and a node-only `SelectorSet` returns no edges.
pub fn selector_match(file: &OmtsFile, selectors: &SelectorSet) -> SelectorMatchResult {
    let mut result = SelectorMatchResult::default();

    if selectors.is_empty() {
        // Universal match: return all nodes and edges.
        result.node_indices = (0..file.nodes.len()).collect();
        result.edge_indices = (0..file.edges.len()).collect();
        return result;
    }

    if selectors.has_node_selectors() {
        for (i, node) in file.nodes.iter().enumerate() {
            if selectors.matches_node(node) {
                result.node_indices.push(i);
            }
        }
    }

    if selectors.has_edge_selectors() {
        for (i, edge) in file.edges.iter().enumerate() {
            if selectors.matches_edge(edge) {
                result.edge_indices.push(i);
            }
        }
    }

    result
}

/// Extracts a subgraph based on selector predicates.
///
/// The extraction runs in four sequential phases:
///
/// 1. **Seed scan** — evaluates `selectors` against every node and edge via a
///    linear pass. Produces `seed_nodes: HashSet<NodeIndex>` and
///    `seed_edges: HashSet<EdgeIndex>`.
///
/// 2. **Seed edge resolution** — for each seed edge, adds its source and target
///    to `seed_nodes`. This ensures matched edges contribute endpoints to the
///    BFS frontier.
///
/// 3. **BFS expansion** — starting from `seed_nodes`, performs bounded BFS for
///    `expand` hops (treating the graph as undirected). Complexity: O(V + E)
///    per hop.
///
/// 4. **Induced subgraph assembly** — delegates to [`assemble_subgraph`] to
///    produce the final [`OmtsFile`]. Complexity: O(E).
///
/// # Parameters
///
/// - `graph` — the graph built from `file` via [`crate::graph::build_graph`].
/// - `file` — the source [`OmtsFile`]; provides full node/edge data and header.
/// - `selectors` — the predicate set to match against.
/// - `expand` — number of BFS hops to expand from the seed set (0 = seed +
///   immediate incident elements only).
///
/// # Errors
///
/// Returns [`QueryError::EmptyResult`] when the selector scan matches no nodes
/// and no edges.
pub fn selector_subgraph(
    graph: &OmtsGraph,
    file: &OmtsFile,
    selectors: &SelectorSet,
    expand: usize,
) -> Result<OmtsFile, QueryError> {
    // Fast path: an empty SelectorSet is a universal match — seed all nodes.
    if selectors.is_empty() {
        let all_nodes: HashSet<NodeIndex> = graph.graph().node_indices().collect();
        return assemble_subgraph(graph, file, &all_nodes);
    }

    let mut seed_nodes: HashSet<NodeIndex> = HashSet::new();

    if selectors.has_node_selectors() {
        if can_use_node_type_index(selectors) {
            for node_type in &selectors.node_types {
                for &idx in graph.nodes_of_type(node_type) {
                    seed_nodes.insert(idx);
                }
            }
        } else {
            for node in &file.nodes {
                if selectors.matches_node(node) {
                    if let Some(&idx) = graph.node_index(node.id.as_ref()) {
                        seed_nodes.insert(idx);
                    }
                }
            }
        }
    }

    let mut seed_edge_node_ids: Vec<(String, String)> = Vec::new();
    let mut any_edge_matched = false;

    if selectors.has_edge_selectors() {
        if can_use_edge_type_index(selectors) {
            let g = graph.graph();
            for edge_type in &selectors.edge_types {
                for &edge_idx in graph.edges_of_type(edge_type) {
                    if let Some((src, tgt)) = g.edge_endpoints(edge_idx) {
                        any_edge_matched = true;
                        if let (Some(sw), Some(tw)) =
                            (graph.node_weight(src), graph.node_weight(tgt))
                        {
                            seed_edge_node_ids.push((sw.local_id.clone(), tw.local_id.clone()));
                        }
                    }
                }
            }
        } else {
            for edge in &file.edges {
                if selectors.matches_edge(edge) {
                    any_edge_matched = true;
                    seed_edge_node_ids.push((edge.source.to_string(), edge.target.to_string()));
                }
            }
        }
    }

    if seed_nodes.is_empty() && !any_edge_matched {
        return Err(QueryError::EmptyResult);
    }

    for (source_id, target_id) in &seed_edge_node_ids {
        if let Some(&idx) = graph.node_index(source_id.as_str()) {
            seed_nodes.insert(idx);
        }
        if let Some(&idx) = graph.node_index(target_id.as_str()) {
            seed_nodes.insert(idx);
        }
    }

    let mut visited: HashSet<NodeIndex> = seed_nodes.clone();
    let mut queue: VecDeque<(NodeIndex, usize)> =
        seed_nodes.iter().map(|&idx| (idx, 0usize)).collect();

    let g = graph.graph();

    while let Some((current, hops)) = queue.pop_front() {
        if hops >= expand {
            continue;
        }
        let next_hops = hops + 1;

        for edge_ref in g.edges(current) {
            let neighbour = edge_ref.target();
            if !visited.contains(&neighbour) {
                visited.insert(neighbour);
                queue.push_back((neighbour, next_hops));
            }
        }
        for edge_ref in g.edges_directed(current, petgraph::Direction::Incoming) {
            let neighbour = edge_ref.source();
            if !visited.contains(&neighbour) {
                visited.insert(neighbour);
                queue.push_back((neighbour, next_hops));
            }
        }
    }

    assemble_subgraph(graph, file, &visited)
}

/// Returns `true` when `node_types` is the only non-empty node-applicable
/// selector group, allowing the type index to replace a full linear scan.
fn can_use_node_type_index(ss: &SelectorSet) -> bool {
    !ss.node_types.is_empty()
        && ss.label_keys.is_empty()
        && ss.label_key_values.is_empty()
        && ss.identifier_schemes.is_empty()
        && ss.identifier_scheme_values.is_empty()
        && ss.jurisdictions.is_empty()
        && ss.names.is_empty()
}

/// Returns `true` when `edge_types` is the only non-empty edge-applicable
/// selector group, allowing the type index to replace a full linear scan.
fn can_use_edge_type_index(ss: &SelectorSet) -> bool {
    !ss.edge_types.is_empty() && ss.label_keys.is_empty() && ss.label_key_values.is_empty()
}

/// Assembles an [`OmtsFile`] from a set of included [`NodeIndex`] values.
///
/// Iterates all edges in the graph; includes an edge in the output only if
/// both its source and target are in `index_set`.  Nodes are included in
/// original file order (by `data_index`) to keep output deterministic.
///
/// The `reporting_entity` header field is preserved only when the referenced
/// node is present in `index_set`; otherwise it is set to `None`.
fn assemble_subgraph(
    graph: &OmtsGraph,
    file: &OmtsFile,
    index_set: &HashSet<NodeIndex>,
) -> Result<OmtsFile, QueryError> {
    let g = graph.graph();

    let mut included_data_indices: HashSet<usize> = HashSet::with_capacity(index_set.len());
    for &idx in index_set {
        if let Some(weight) = graph.node_weight(idx) {
            included_data_indices.insert(weight.data_index);
        }
    }

    let nodes: Vec<crate::structures::Node> = file
        .nodes
        .iter()
        .enumerate()
        .filter(|(i, _)| included_data_indices.contains(i))
        .map(|(_, node)| node.clone())
        .collect();

    let mut included_edge_data_indices: HashSet<usize> = HashSet::new();
    for &node_idx in index_set {
        for edge_ref in g.edges(node_idx) {
            if index_set.contains(&edge_ref.target()) {
                included_edge_data_indices.insert(edge_ref.weight().data_index);
            }
        }
    }

    let edges: Vec<crate::structures::Edge> = file
        .edges
        .iter()
        .enumerate()
        .filter(|(i, _)| included_edge_data_indices.contains(i))
        .map(|(_, edge)| edge.clone())
        .collect();

    let included_node_ids: HashSet<String> = nodes.iter().map(|n| n.id.to_string()).collect();
    let reporting_entity = file.reporting_entity.as_ref().and_then(|re_id| {
        if included_node_ids.contains(&re_id.to_string()) {
            Some(re_id.clone())
        } else {
            None
        }
    });

    Ok(OmtsFile {
        omtsf_version: file.omtsf_version.clone(),
        snapshot_date: file.snapshot_date.clone(),
        file_salt: file.file_salt.clone(),
        disclosure_scope: file.disclosure_scope.clone(),
        previous_snapshot_ref: file.previous_snapshot_ref.clone(),
        snapshot_sequence: file.snapshot_sequence,
        reporting_entity,
        nodes,
        edges,
        extra: file.extra.clone(),
    })
}
