#[derive(Debug)]

pub(crate) struct Node {
    parent: Option<NodeHandle>,
    next_sibling: Option<NodeHandle>,
    previous_sibling: Option<NodeHandle>,
    first_child: Option<NodeHandle>,
    last_child: Option<NodeHandle>,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct NodeHandle(pub(crate) usize);

#[derive(Debug)]
pub struct Tree {
    pub(crate) nodes: Vec<Node>,
    available_nodes: Vec<NodeHandle>,
}

impl Tree {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            available_nodes: Vec::new(),
        }
    }

    fn node(&self, node_handle: NodeHandle) -> &Node {
        &self.nodes[node_handle.0]
    }

    fn node_mut(&mut self, node_handle: NodeHandle) -> &mut Node {
        &mut self.nodes[node_handle.0]
    }

    /// Adds a child as the last child of the parent.
    pub fn add(&mut self, parent: Option<NodeHandle>) -> NodeHandle {
        let new_node = Node {
            parent,
            next_sibling: None,
            previous_sibling: None,
            first_child: None,
            last_child: None,
        };

        let new_handle = if let Some(free_node) = self.available_nodes.pop() {
            self.nodes[free_node.0] = new_node;
            free_node
        } else {
            self.nodes.push(new_node);
            NodeHandle(self.nodes.len() - 1)
        };

        if let Some(parent_handle) = parent {
            let parent = self.node(parent_handle);
            if let Some(last_child) = parent.last_child {
                self.node_mut(last_child).next_sibling = Some(new_handle);
                self.node_mut(new_handle).previous_sibling = Some(last_child);
            }

            let parent = self.node_mut(parent_handle);
            if parent.first_child.is_none() {
                parent.first_child = Some(new_handle);
            }
            parent.last_child = Some(new_handle);
        }

        new_handle
    }

    fn remove_descendents(
        nodes: &Vec<Node>,
        available_nodes: &mut Vec<NodeHandle>,
        node: NodeHandle,
    ) {
        // Iterate all descendents and push their index to the free indices.
        // No need to unhook them as their parent will be disconnected.
        let iterator = TreeSiblingIterator {
            node: Some(node),
            nodes,
        };
        for child in iterator {
            available_nodes.push(child);
            Self::remove_descendents(nodes, available_nodes, child);
        }
    }

    pub fn remove(&mut self, node_handle: NodeHandle) {
        let node = self.node(node_handle);

        // Only remove the node if it's active
        let next_sibling = node.next_sibling;
        let previous_sibling = node.previous_sibling;
        let parent = node.parent;
        if let Some(parent) = parent {
            // Connect parent children
            let parent = self.node_mut(parent);
            if parent.first_child == Some(node_handle) {
                parent.first_child = next_sibling;
            }
            if parent.last_child == Some(node_handle) {
                parent.last_child = previous_sibling;
            }

            // Connect siblings
            if let Some(next_sibling) = next_sibling {
                self.node_mut(next_sibling).previous_sibling = previous_sibling;
            }

            if let Some(previous_sibling) = previous_sibling {
                self.node_mut(previous_sibling).next_sibling = next_sibling;
            }
        }
        Self::remove_descendents(&self.nodes, &mut self.available_nodes, node_handle);
        self.available_nodes.push(node_handle);
    }

    pub fn child_iter(&self, node: NodeHandle) -> TreeSiblingIterator {
        TreeSiblingIterator {
            node: self.nodes[node.0].first_child,
            nodes: &self.nodes,
        }
    }

    pub fn sibling_iter(&self, node: Option<NodeHandle>) -> TreeSiblingIterator {
        TreeSiblingIterator {
            node,
            nodes: &self.nodes,
        }
    }
}

pub struct TreeSiblingIterator<'a> {
    node: Option<NodeHandle>,
    nodes: &'a Vec<Node>,
}

impl<'a> Iterator for TreeSiblingIterator<'a> {
    // we will be counting with usize
    type Item = NodeHandle;

    // next() is the only required method
    fn next(&mut self) -> Option<NodeHandle> {
        self.node.map(|n| {
            let node = &self.nodes[n.0];
            self.node = node.next_sibling;
            n
        })
    }
}
