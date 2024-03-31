use std::{cmp::Reverse, collections::{BinaryHeap, HashMap}};

trait HuffmanBaseNode: Eq + PartialEq + PartialOrd {
    fn is_leaf(&self) -> bool;
    fn weight(&self) -> u32;
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Clone)]
struct HuffmanLeafNode {
    element: char,
    weight: u32,
}

impl HuffmanLeafNode {
    fn new(element: char, weight: u32) -> Self {
        HuffmanLeafNode { element, weight }
    }

    fn element(&self) -> char {
        self.element
    }
}

impl HuffmanBaseNode for HuffmanLeafNode {
    fn weight(&self) -> u32 {
        self.weight
    }
    fn is_leaf(&self) -> bool {
        true
    }
}

impl HuffmanBaseNode for HuffmanNode {
    fn is_leaf(&self) -> bool {
        match self {
            HuffmanNode::Leaf(_) => true,
            HuffmanNode::Internal(_) => false,
        }
    }

    fn weight(&self) -> u32 {
        match self {
            HuffmanNode::Leaf(leaf) => leaf.weight(),
            HuffmanNode::Internal(internal) => internal.weight(),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Clone)]
struct HuffmanInternalNode {
    weight: u32,
    left: HuffmanNode,
    right: HuffmanNode,
}

impl HuffmanInternalNode {
    fn new(left: HuffmanNode, right: HuffmanNode, weight: u32) -> Self {
        HuffmanInternalNode {
            left,
            right,
            weight,
        }
    }

    fn left(&self) -> &HuffmanNode {
        &self.left
    }

    fn right(&self) -> &HuffmanNode {
        &self.right
    }
}

impl HuffmanBaseNode for HuffmanInternalNode {
    fn weight(&self) -> u32 {
        self.weight
    }
    fn is_leaf(&self) -> bool {
        false
    }
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Clone)]
enum HuffmanNode {
    Leaf(Box<HuffmanLeafNode>),
    Internal(Box<HuffmanInternalNode>),
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
struct HuffmanTree {
    root: Box<HuffmanNode>,
}

impl HuffmanTree {
    fn new(root: Box<HuffmanNode>) -> Self {
        HuffmanTree { root }
    }

    fn merge(left: Box<HuffmanNode>, right: Box<HuffmanNode>, weight: u32) -> Self {
        let root = HuffmanInternalNode::new(*left, *right, weight);
        HuffmanTree::new(Box::new(HuffmanNode::Internal(Box::new(root))))
    }

    fn root(&self) -> &HuffmanNode {
        &*self.root
    }
}

type MinHeapTuple = Reverse<(u32, HuffmanTree)>;

fn build_tree(freq_map: &HashMap<char, u32>) -> HuffmanTree {
    let mut heap = BinaryHeap::<MinHeapTuple>::new();
    for (c, freq) in freq_map {
        let leaf = HuffmanLeafNode::new(*c, *freq);
        let tree = HuffmanTree::new(Box::new(HuffmanNode::Leaf(Box::new(leaf))));
        heap.push(Reverse((*freq, tree)));
    }

    build_tree_from_heap(heap)
}

fn build_tree_from_heap(mut heap: BinaryHeap<MinHeapTuple>) -> HuffmanTree {
    while heap.len() > 1 {
        let left = heap.pop().unwrap();
        let right = heap.pop().unwrap();

        let root = HuffmanTree::merge(
            Box::new(left.0.1.root().clone()),
            Box::new(right.0.1.root().clone()),
            left.0.0 + right.0.0,
        );

        heap.push(Reverse((root.root().weight(), root)));
    }

    heap.pop().unwrap().0.1
}

pub fn build_code_lookup(freq_map: &HashMap<char, u32>) -> HashMap<char, String> {
    let mut code_lookup = HashMap::new();
    let huffman_tree = build_tree(freq_map);
    let mut code = String::new();
    build_code_lookup_recursive(&mut code_lookup, huffman_tree.root(), &mut code);

    code_lookup
}

fn build_code_lookup_recursive(
    code_lookup: &mut HashMap<char, String>,
    node: &HuffmanNode,
    code: &mut String,
) {
    match node {
        HuffmanNode::Leaf(leaf) => {
            code_lookup.insert(leaf.element(), code.to_string());
        },
        HuffmanNode::Internal(internal) => {
            code.push('0');
            build_code_lookup_recursive(code_lookup, internal.left(), code);
            code.pop();

            code.push('1');
            build_code_lookup_recursive(code_lookup, internal.right(), code);
            code.pop();
        },
    }
}
