use std::{
    path::Path,
    sync::{LazyLock, OnceLock},
};

use anyhow::{anyhow, Result};
use petgraph::graph::DiGraph;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize)]
pub struct EngineConfig {
    graph: DiGraph<Node, ()>,
    nats: NatsConfig,
    plugin: PluginConfig,
}

// TODO
// -- update to encompass async_nats::ToServerAddrs
// -- naming
#[derive(Clone, Debug, Deserialize)]
pub struct NatsConfig {
    url: String,
    enable_execution_thread: bool,
    enable_watcher_thread: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct PluginConfig {
    wasi: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Node {
    pub name: String,
    pub bucket: String,
    pub object: String,
    pub namespace: String,
    pub hash: String,
}

// #[derive(Clone, Debug, Deserialize, Serialize)]
// struct Edge {
//     source: String,
//     target: String,
// }

// #[derive(Clone, Debug, Deserialize, Serialize)]
// struct GraphConfig {
//     nodes: Vec<Node>,
//     edges: Vec<Edge>,
// }

static CONFIG: OnceLock<EngineConfig> = OnceLock::new();

// fn build_graph_config(config: GraphConfig) -> Result<DiGraph<Node, ()>> {
//     let mut graph = DiGraph::new();

//     let mut indices = std::collections::HashMap::new();
//     for node in &config.nodes {
//         let index = graph.add_node(node.clone());
//         indices.insert(node.name.clone(), index);
//     }

//     for edge in &config.edges {
//         let source_index = *indices
//             .get(&edge.source)
//             .ok_or(anyhow!("failed to get source index"))?;
//         let target_index = *indices
//             .get(&edge.target)
//             .ok_or(anyhow!("failed to get target index"))?;
//         graph.add_edge(source_index, target_index, ());
//     }

//     Ok(graph)
// }

pub fn require_config(path: &str) -> Result<()> {
    let raw_config = std::fs::read_to_string(path)?;
    let config = serde_yaml::from_str::<EngineConfig>(&raw_config)?;
    CONFIG.set(config).ok();

    Ok(())
}

pub fn get_config() -> EngineConfig {
    CONFIG.get().unwrap().clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_graph_from_config() {
        // TODO--
        let graph_config = serde_yaml::from_str::<GraphConfig>(
            "
            nodes:
              - name: make_auth_call
                bucket: wasm
                object: make-auth-call
                namespace: main
                hash: 123abc
              - name: create_pop_token
                bucket: wasm
                object: create-pop-token
                namespace: create_pop_token
                hash: 123abd
              - name: create_jti
                bucket: wasm
                object: create-jti
                namespace: create_jti
                hash: 123abe

            edges:
              - source: make_auth_call
                target: create_pop_token
              - source: create_pop_token
                target: create_jti
        ",
        )
        .unwrap();

        let expected_graph = serde_json::from_str::<DiGraph<Node, ()>>(
            r#"{
            "nodes": [
                {
                    "name": "make_auth_call",
                    "bucket": "wasm",
                    "object": "make-auth-call",
                    "namespace": "main",
                    "hash": "123abc"
                },
                {
                    "name": "create_pop_token",
                    "bucket": "wasm",
                    "object": "create-pop-token",
                    "namespace": "create_pop_token",
                    "hash": "123abd"
                },
                {
                    "name": "create_jti",
                    "bucket": "wasm",
                    "object": "create-jti",
                    "namespace": "create_jti",
                    "hash": "123abe"
                }
            ],
            "node_holes": [],
            "edge_property": "directed",
            "edges": [
                [
                    0,
                    1,
                    null
                ],
                [
                    1,
                    2,
                    null
                ]
            ]
            }"#,
        )
        .unwrap();

        let actual_graph = build_graph_from_config(graph_config).unwrap();

        let actual_value = serde_json::to_value(actual_graph).unwrap();
        let expected_value = serde_json::to_value(expected_graph).unwrap();
        assert_eq!(actual_value, expected_value);
    }
}
