use std::collections::{HashMap, HashSet};
use petgraph::graph::NodeIndex;
use petgraph::visit::EdgeRef;
use cognition_core::{CognitionResult, EdgeType, NodeId};
use crate::graph::CognitiveGraph;

fn get_multiplier(edge_type: EdgeType) -> f32 {
    match edge_type {
        EdgeType::AoCausal => 2.0,
        EdgeType::Entity => 1.5,
        EdgeType::Transition => 1.8,
        EdgeType::Temporal => 1.0,
        EdgeType::Semantic => 0.8,
        _ => 1.0,
    }
}

/// Parameters for the Spreading Activation algorithm.
/// Based on Baddeley's Working Memory Model and Cognitive Cycle-Guards.
#[derive(Debug, Clone)]
pub struct ActivationParams {
    /// Decay factor (delta) - Energy lost at each hop (e.g., 0.8)
    pub delta: f32,             
    /// Saturation Threshold (A_max) - Prevents sum divergence (e.g., 2.0)
    pub a_max: f32,             
    /// Convergence threshold - Stop if max delta is below this (e.g., 0.01)
    pub epsilon: f32,           
    /// Maximum propagation steps (fallback safety)
    pub max_steps: usize,       
    /// Global Firing Quota - Max times a node can broadcast per query (e.g., 3)
    pub max_fires: u8,          
}

impl Default for ActivationParams {
    fn default() -> Self {
        Self {
            delta: 0.8,
            a_max: 2.0,
            epsilon: 0.01,
            max_steps: 10,
            max_fires: 3,
        }
    }
}

impl CognitiveGraph {
    /// Executes SUM-based Spreading Activation over the Knowledge Graph.
    /// 
    /// # Arguments
    /// * `seeds` - The initial activated nodes from Semantic/BM25 search (NodeId -> Initial Score)
    /// * `params` - Hyperparameters for the cognitive simulation
    pub async fn spreading_activation(
        &self,
        seeds: HashMap<NodeId, f32>,
        params: &ActivationParams,
    ) -> CognitionResult<HashMap<NodeId, f32>> {
        let graph_guard = self.inner.read().await;
        let indices_guard = self.node_indices.read().await;

        // State Init
        // A(v, t): Current activation scores
        let mut activations: HashMap<NodeIndex, f32> = HashMap::new();
        // Cycle Guard 2: Flobal Firing Quota tracker
        let mut firing_counts: HashMap<NodeIndex, u8> = HashMap::new();
        // Cycle Guard 1: Local Refractory Period tracker
        let mut just_fired: HashSet<NodeIndex> = HashSet::new();

        // Map seeds UUIDs to internal petgraph NodeIndices
        for (id, initial_score) in seeds {
            if let Some(&idx) = indices_guard.get(&id) {
                // Apply Saturation Threshold right at initialization
                activations.insert(idx, initial_score.min(params.a_max));
            }
        }

        // Propagation Loop
        for _step in 0..params.max_steps {
            let mut next_activations = activations.clone();
            let mut next_just_fired = HashSet::new();
            let mut max_delta: f32 = 0.0;
            let mut has_fired = false;

            // Identify nodes eligible to fire (The "Attention" mechanism)
            let mut firing_nodes = Vec::new();
            for (&u, &a_u) in &activations {
                // Must have energy AND not be in Refractory Period
                if a_u > 0.0 && !just_fired.contains(&u) {
                    let count = firing_counts.get(&u).copied().unwrap_or(0);
                    if count < params.max_fires {
                        firing_nodes.push(u);
                    }
                }
            }

            // Early exit if no nodes can fire
            if firing_nodes.is_empty() {
                break;
            }

            // Broadcast energy (SUM Logic)
            for u in firing_nodes {
                has_fired = true;
                let a_u = activations[&u];

                // Mark as fired for Quota and Refractory tracking
                *firing_counts.entry(u).or_insert(0) += 1;
                next_just_fired.insert(u);

                // Iterate over all outgoing edges from `u`
                for edge in graph_guard.edges(u) {
                    let v = edge.target();
                    let edge_data = edge.weight();

                    // Energy = A(u,t) * w(u,v) * mu(edge_type) * delta
                    let mu = get_multiplier(edge_data.edge_type);
                    let energy = a_u * edge_data.weight * mu * params.delta;

                    // Accumulate evidence
                    let current_v = *next_activations.get(&v).unwrap_or(&0.0);
                    let mut new_v = current_v + energy;

                    new_v = new_v.min(params.a_max); // Apply Saturation Threshold
                    next_activations.insert(v, new_v);
                }
            }

            // Covergence check
            for (&v, &new_a) in &next_activations {
                let old_a = *activations.get(&v).unwrap_or(&0.0);
                let diff = (new_a - old_a).abs();
                if diff > max_delta {
                    max_delta = diff;
                }
            }

            activations = next_activations;
            just_fired = next_just_fired;

            if !has_fired || max_delta < params.epsilon {
                break;
            }
        }
    
    let mut result = HashMap::new();
    for (idx, score) in activations {
        if let Some(node) = graph_guard.node_weight(idx) {
            result.insert(node.id, score);
        }
    }

    Ok(result) // Return the final activated nodes mapping (NodeId -> Activation Score)
    
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{EdgeType, EdgeData};
    use cognition_core::{MemoryNode, NetworkType};
    use std::collections::HashMap;

    fn create_test_node(content: &str, network: NetworkType) -> MemoryNode {
        MemoryNode::new(network, content.to_string())
    }

    #[tokio::test]
    async fn test_sum_accumulation_logic() {
        // Scenario: 
        // Node A ("Colleague") and Node B ("Interest") both point to Node C ("Project")
        // In Hindsight (MAX): Score of C = max(contributions)
        // In CogMem (SUM): Score of C = sum(contributions)

        let graph = CognitiveGraph::new();
        let n_colleague = create_test_node("Colleague", NetworkType::World);
        let n_interest = create_test_node("AI Research", NetworkType::Opinion);
        let n_project = create_test_node("Project Cognition", NetworkType::Experience);

        let id_a = n_colleague.id;
        let id_b = n_interest.id;
        let id_c = n_project.id;

        graph.add_node(n_colleague).await.unwrap();
        graph.add_node(n_interest).await.unwrap();
        graph.add_node(n_project).await.unwrap();

        // Edge A -> C (weight 0.5)
        graph.add_edge(id_a, id_c, EdgeData::new(EdgeType::Entity, 0.5, 1.0)).await.unwrap();
        // Edge B -> C (weight 0.5)
        graph.add_edge(id_b, id_c, EdgeData::new(EdgeType::Semantic, 0.5, 1.0)).await.unwrap();

        let mut seeds = HashMap::new();
        seeds.insert(id_a, 0.8);
        seeds.insert(id_b, 0.8);

        let params = ActivationParams {
            delta: 1.0,      // No decay for simple test
            a_max: 2.0,      // High saturation
            epsilon: 0.01,
            max_steps: 2,
            max_fires: 1,
        };

        let results = graph.spreading_activation(seeds, &params).await.unwrap();

        // Calculation:
        // Contribution from A = 0.8 * 0.5 * 1.0 (weight) * 1.0 (delta) = 0.4
        // Contribution from B = 0.8 * 0.5 * 1.0 (weight) * 1.0 (delta) = 0.4
        // Total at C = 0.4 + 0.4 = 0.8
        // If it were MAX logic, C would only be 0.4.

        let score_c = *results.get(&id_c).unwrap_or(&0.0);
        assert!(score_c > 0.7, "SUM logic failed: score should be 0.8, got {}", score_c);
    }

    #[tokio::test]
    async fn test_saturation_guard() {
        // Scenario: Multiple strong inputs that should exceed 2.0
        // Result: Must be clipped at a_max (2.0)
        
        let graph = CognitiveGraph::new();
        let n_source1 = create_test_node("Source 1", NetworkType::World);
        let n_source2 = create_test_node("Source 2", NetworkType::World);
        let n_target = create_test_node("Target", NetworkType::World);
        
        let id1 = n_source1.id;
        let id2 = n_source2.id;
        let id_t = n_target.id;

        graph.add_node(n_source1).await.unwrap();
        graph.add_node(n_source2).await.unwrap();
        graph.add_node(n_target).await.unwrap();

        // Very strong edges
        graph.add_edge(id1, id_t, EdgeData::new(EdgeType::Causal, 1.0, 2.0)).await.unwrap();
        graph.add_edge(id2, id_t, EdgeData::new(EdgeType::Causal, 1.0, 2.0)).await.unwrap();

        let mut seeds = HashMap::new();
        seeds.insert(id1, 1.0);
        seeds.insert(id2, 1.0);

        let params = ActivationParams {
            a_max: 1.5, // Tight saturation
            ..Default::default()
        };

        let results = graph.spreading_activation(seeds, &params).await.unwrap();
        let score_t = *results.get(&id_t).unwrap_or(&0.0);
        
        assert!(score_t <= 1.5, "Saturation guard failed: score {} exceeded a_max 1.5", score_t);
    }

    #[tokio::test]
    async fn test_firing_quota_guard() {
        // Scenario: A cycle A <-> B. 
        // Without quota, they would fire forever.
        // With max_fires = 2, they should stop.
        
        let graph = CognitiveGraph::new();
        let n_a = create_test_node("Node A", NetworkType::World);
        let n_b = create_test_node("Node B", NetworkType::World);
        let id_a = n_a.id;
        let id_b = n_b.id;

        graph.add_node(n_a).await.unwrap();
        graph.add_node(n_b).await.unwrap();

        graph.add_edge(id_a, id_b, EdgeData::new(EdgeType::Semantic, 0.9, 1.0)).await.unwrap();
        graph.add_edge(id_b, id_a, EdgeData::new(EdgeType::Semantic, 0.9, 1.0)).await.unwrap();

        let mut seeds = HashMap::new();
        seeds.insert(id_a, 1.0);

        let params = ActivationParams {
            max_steps: 100, // Large steps to force quota limit
            max_fires: 2,   // Each node can only fire twice
            delta: 0.9,
            ..Default::default()
        };

        let results = graph.spreading_activation(seeds, &params).await.unwrap();
        
        // If it didn't crash or run forever, the guard worked.
        assert!(results.contains_key(&id_a));
        assert!(results.contains_key(&id_b));
        println!("Cycle test completed safely.");
    }
}