use crate::image::{Color, Image};
use crate::palette::{Palette, PaletteGenerator};

pub struct Octree;

#[derive(Debug, Clone)]
pub struct OctreeParams {
    pub n_of_colors: u32,
}

struct OctreeNode {
    color_sum: (u64, u64, u64),
    pixel_count: u64,
    children: [Option<usize>; 8],
    is_leaf: bool,
}

struct OctreeTree {
    nodes: Vec<OctreeNode>,
    levels: [Vec<usize>; 8], // Track nodes at level 0..=7. Level 0 is the root.
    leaf_count: usize,
}

impl OctreeTree {
    fn new() -> Self {
        let mut tree = OctreeTree {
            nodes: Vec::new(),
            levels: Default::default(),
            leaf_count: 0,
        };
        // Create root node
        tree.nodes.push(OctreeNode {
            color_sum: (0, 0, 0),
            pixel_count: 0,
            children: [None; 8],
            is_leaf: false,
        });
        tree.levels[0].push(0);
        tree
    }

    fn insert(&mut self, color: [u8; 3], max_colors: usize) {
        let mut curr_idx = 0;
        
        self.nodes[curr_idx].color_sum.0 += color[0] as u64;
        self.nodes[curr_idx].color_sum.1 += color[1] as u64;
        self.nodes[curr_idx].color_sum.2 += color[2] as u64;
        self.nodes[curr_idx].pixel_count += 1;

        for depth in 0..8 {
            if self.nodes[curr_idx].is_leaf {
                break;
            }

            let r = color[0];
            let g = color[1];
            let b = color[2];
            let bit = 7 - depth;
            let child_idx = (((r >> bit) & 1) << 2) | (((g >> bit) & 1) << 1) | ((b >> bit) & 1);
            let child_idx = child_idx as usize;

            let next_idx = match self.nodes[curr_idx].children[child_idx] {
                Some(idx) => idx,
                None => {
                    let new_idx = self.nodes.len();
                    let is_leaf = depth == 7;
                    self.nodes.push(OctreeNode {
                        color_sum: (0, 0, 0),
                        pixel_count: 0,
                        children: [None; 8],
                        is_leaf,
                    });
                    self.nodes[curr_idx].children[child_idx] = Some(new_idx);
                    if is_leaf {
                        self.leaf_count += 1;
                    } else {
                        self.levels[depth + 1].push(new_idx);
                    }
                    new_idx
                }
            };

            self.nodes[next_idx].color_sum.0 += color[0] as u64;
            self.nodes[next_idx].color_sum.1 += color[1] as u64;
            self.nodes[next_idx].color_sum.2 += color[2] as u64;
            self.nodes[next_idx].pixel_count += 1;

            curr_idx = next_idx;
        }

        while self.leaf_count > max_colors {
            self.reduce();
        }
    }

    fn reduce(&mut self) {
        // Find deepest level containing nodes that have children
        for depth in (0..8).rev() {
            if self.levels[depth].is_empty() {
                continue;
            }

            // Find a node that has children
            let mut found_node_idx = None;
            let mut min_pixels = u64::MAX;

            for &node_idx in &self.levels[depth] {
                let has_children = self.nodes[node_idx].children.iter().any(|c| c.is_some());
                if has_children {
                    // Pick node with minimum pixels to merge
                    if self.nodes[node_idx].pixel_count < min_pixels {
                        min_pixels = self.nodes[node_idx].pixel_count;
                        found_node_idx = Some(node_idx);
                    }
                }
            }

            if let Some(node_idx) = found_node_idx {
                // Merge children
                let mut sum_r = 0;
                let mut sum_g = 0;
                let mut sum_b = 0;
                let mut count = 0;
                let mut child_leaves_removed = 0;

                for i in 0..8 {
                    if let Some(child_idx) = self.nodes[node_idx].children[i] {
                        let child = &self.nodes[child_idx];
                        sum_r += child.color_sum.0;
                        sum_g += child.color_sum.1;
                        sum_b += child.color_sum.2;
                        count += child.pixel_count;
                        if child.is_leaf {
                            child_leaves_removed += 1;
                        }
                        self.nodes[node_idx].children[i] = None;
                    }
                }

                self.nodes[node_idx].color_sum = (sum_r, sum_g, sum_b);
                self.nodes[node_idx].pixel_count = count;
                self.nodes[node_idx].is_leaf = true;

                self.leaf_count = self.leaf_count + 1 - child_leaves_removed;
                return;
            }
        }
    }

    fn collect_leaves(&self, node_idx: usize, palette: &mut Vec<[u8; 3]>) {
        let node = &self.nodes[node_idx];
        if node.is_leaf {
            if node.pixel_count > 0 {
                let r = (node.color_sum.0 / node.pixel_count) as u8;
                let g = (node.color_sum.1 / node.pixel_count) as u8;
                let b = (node.color_sum.2 / node.pixel_count) as u8;
                palette.push([r, g, b]);
            }
        } else {
            for i in 0..8 {
                if let Some(child_idx) = node.children[i] {
                    self.collect_leaves(child_idx, palette);
                }
            }
        }
    }
}

impl PaletteGenerator for Octree {
    type Params = OctreeParams;

    fn name(&self) -> &'static str {
        "octree"
    }

    fn calculate(&self, image: &Image, params: &Self::Params) -> Palette {
        let colors_slice = image.colors();
        if colors_slice.is_empty() || params.n_of_colors == 0 {
            return Palette { colors: Vec::new() };
        }

        let max_colors = params.n_of_colors as usize;
        let mut tree = OctreeTree::new();

        for color in colors_slice {
            tree.insert([color.r, color.g, color.b], max_colors);
        }

        let mut raw_palette = Vec::new();
        tree.collect_leaves(0, &mut raw_palette);

        let colors = raw_palette
            .into_iter()
            .map(|[r, g, b]| Color { r, g, b, a: 255 })
            .collect();

        Palette { colors }
    }
}
