use gazpatcho::model::{Node, Patch};
use gazpatcho::report::Report;
use std::collections::{HashMap, HashSet};

pub struct Diff {
    pub added_nodes: Vec<Node>,
    pub updated_nodes: Vec<Node>,
    pub removed_nodes: Vec<Node>,
    pub added_patches: Vec<Patch>,
    pub removed_patches: Vec<Patch>,
}

impl Diff {
    pub fn new(old_report: Option<&Report>, new_report: &Report) -> Self {
        if old_report.is_none() {
            return Self {
                added_nodes: new_report.nodes.clone(),
                updated_nodes: vec![],
                removed_nodes: vec![],
                added_patches: new_report.patches.clone(),
                removed_patches: vec![],
            };
        }

        let old_report = old_report.unwrap();

        let old_nodes_by_id: HashMap<_, _> =
            old_report.nodes.iter().map(|n| (n.id.clone(), n)).collect();
        let new_nodes_by_id: HashMap<_, _> =
            old_report.nodes.iter().map(|n| (n.id.clone(), n)).collect();

        let added_nodes: Vec<_> = new_nodes_by_id
            .iter()
            .filter(|(id, _)| !old_nodes_by_id.contains_key(*id))
            .map(|(_, n)| *n)
            .cloned()
            .collect();

        let updated_nodes: Vec<_> = new_nodes_by_id
            .iter()
            .filter(|(id, new_n)| {
                if let Some(old_n) = old_nodes_by_id.get(*id) {
                    *new_n != old_n
                } else {
                    false
                }
            })
            .map(|(_, n)| *n)
            .cloned()
            .collect();

        let removed_nodes: Vec<_> = old_nodes_by_id
            .iter()
            .filter(|(id, _)| !new_nodes_by_id.contains_key(*id))
            .map(|(_, n)| *n)
            .cloned()
            .collect();

        let old_patches: HashSet<_> = old_report.patches.iter().collect();
        let new_patches: HashSet<_> = new_report.patches.iter().collect();

        let added_patches = new_patches
            .difference(&old_patches)
            .cloned()
            .cloned()
            .collect();

        let removed_patches = old_patches
            .difference(&new_patches)
            .cloned()
            .cloned()
            .collect();

        Self {
            added_nodes,
            updated_nodes,
            removed_nodes,
            added_patches,
            removed_patches,
        }
    }
}
