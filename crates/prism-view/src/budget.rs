//! View budgets — analogue of pack token budgets.

#[derive(Debug, Clone, Copy)]
pub struct ViewBudget {
    pub max_nodes: usize,
    pub max_edges: usize,
}

pub const DEFAULT_MAX_NODES: usize = 80;
pub const DEFAULT_MAX_EDGES: usize = 160;

impl Default for ViewBudget {
    fn default() -> Self {
        Self {
            max_nodes: DEFAULT_MAX_NODES,
            max_edges: DEFAULT_MAX_EDGES,
        }
    }
}

impl ViewBudget {
    pub fn clamp(self) -> Self {
        Self {
            max_nodes: self.max_nodes.clamp(1, 2000),
            max_edges: self.max_edges.clamp(0, 5000),
        }
    }
}
