
// Utility for packing characters into the smallest size atlas possible.
// Note that the problem is the 2 dimensional version of the Bin Packing
// problem, which is essentially NP-hard. The algorithm used is a best-effort
// algorithm based on: https://codeincomplete.com/posts/bin-packing/.
// The algorithm is represented in a generic manner, it could be used for any
// kind of 2D packing, but the exposed API is for rendered glyphs explicitly.

use std::cmp::Ordering;
use std::collections::HashMap;
use std::hash::Hash;
use std::rc::Rc;
use std::cell::RefCell;

/// The packer algorithm itself.
pub(crate) fn bin_pack<
    /// The type being passed as input.
    T,
    /// The key type.
    K: Eq + Hash,
    /// The size selector function.
    FS: FnMut(&T) -> (usize, usize),
    /// The ordering function.
    FO: FnMut(&(usize, usize), &(usize, usize)) -> Ordering,
    /// The key selector function.
    FK: FnMut(&T) -> K,
>(to_pack: impl Iterator<Item = T>,
    mut size_f: FS, mut ordering_f: FO, mut key_f: FK) -> PackResult<K> {
    let mut to_pack: Vec<_> = to_pack.collect();
    to_pack.sort_by(|a, b| ordering_f(&size_f(a), &size_f(b)).reverse());

    let (w, h) = to_pack.first().map(|i| size_f(i)).unwrap_or((0, 0));
    let mut packer = Packer::new(w, h);

    let mut items = HashMap::new();

    for e in to_pack {
        let (w, h) = size_f(&e);
        let k = key_f(&e);
        let rect = packer.fit(w, h);
        items.insert(k, rect);
    }

    let width = packer.root.borrow().width;
    let height = packer.root.borrow().height;
    PackResult{
        width, height, items,
    }
}

/// Returned by the packing operation to summate the results.
pub struct PackResult<K> {
    /// The required width to fit in every entry.
    pub width: usize,
    /// The required height to fit in every entry.
    pub height: usize,
    /// The map from the entry key to it's fit rectangle.
    pub items: HashMap<K, Rect>,
}

/// The backing data-structure to the packing algorithm.
struct Packer {
    /// The root node of the packer.
    root: Rc<RefCell<Node>>,
}

impl Packer {
    /// Creates an empty packer.
    fn new(w: usize, h: usize) -> Self {
        Self{
            root: Rc::new(RefCell::new(Node::new(0, 0, w, h))),
        }
    }

    /// Tries to fit in a block.
    fn fit(&mut self, w: usize, h: usize) -> Rect {
        let node = if let Some(node) = self.find_node(&self.root, w, h) {
                self.split_node(&node, w, h)
            }
            else {
                self.grow_node(w, h)
            };
        let node = node.borrow();
        Rect{
            x: node.x,
            y: node.y,
            width: node.width,
            height: node.height,
        }
    }

    /// Finds the first fitting node, or none in the tree.
    fn find_node(&self, root: &Rc<RefCell<Node>>, w: usize, h: usize) -> Option<Rc<RefCell<Node>>> {
        let node = root.borrow();
        if node.occupied {
            let r = self.find_node(&node.right.as_ref().unwrap(), w, h);
            if r.is_some() {
                return r;
            }
            self.find_node(&node.down.as_ref().unwrap(), w, h)
        }
        else if w <= node.width && h <= node.height {
            Some(root.clone())
        }
        else {
            None
        }
    }

    /// Splits and occupies the node.
    fn split_node(&mut self, node: &Rc<RefCell<Node>>, w: usize, h: usize) -> Rc<RefCell<Node>> {
        let mut bnode = node.borrow_mut();
        bnode.occupied = true;
        bnode.down = Some(Rc::new(RefCell::new(Node::new(bnode.x, bnode.y + h, bnode.width, bnode.height - h))));
        bnode.right = Some(Rc::new(RefCell::new(Node::new(bnode.x + w, bnode.y, bnode.width - w, h))));
        node.clone()
    }

    /// Grows the node in size and tries to remain close to a square.
    fn grow_node(&mut self, w: usize, h: usize) -> Rc<RefCell<Node>> {
        let root_w = self.root.borrow().width;
        let root_h = self.root.borrow().height;

        let can_down = w <= root_w;
        let can_right = h <= root_h;

        let should_right = can_right && (root_h > (root_w + w));
        let should_down = can_down && (root_w > (root_h + h));

        if should_right {
            self.grow_right(w, h)
        }
        else if should_down {
            self.grow_down(w, h)
        }
        else if can_right {
            self.grow_right(w, h)
        }
        else if can_down {
            self.grow_down(w, h)
        }
        else {
            panic!("Invalid sorting!");
        }
    }

    /// Grows a node to the right.
    fn grow_right(&mut self, w: usize, h: usize) -> Rc<RefCell<Node>> {
        let root_w = self.root.borrow().width;
        let root_h = self.root.borrow().height;

        let mut root = Node::new(0, 0, root_w + w, root_h);
        root.occupied = true;
        root.down = Some(self.root.clone());
        root.right = Some(Rc::new(RefCell::new(Node::new(root_w, 0, w, root_h))));
        self.root = Rc::new(RefCell::new(root));

        let node = self.find_node(&self.root, w, h).expect("Invalid sorting!");
        self.split_node(&node, w, h)
    }

    /// Grows a node to down.
    fn grow_down(&mut self, w: usize, h: usize) -> Rc<RefCell<Node>> {
        let root_w = self.root.borrow().width;
        let root_h = self.root.borrow().height;

        let mut root = Node::new(0, 0, root_w, root_h + h);
        root.occupied = true;
        root.right = Some(self.root.clone());
        root.down = Some(Rc::new(RefCell::new(Node::new(0, root_h, root_w, h))));
        self.root = Rc::new(RefCell::new(root));

        let node = self.find_node(&self.root, w, h).expect("Invalid sorting!");
        self.split_node(&node, w, h)
    }
}

/// Represents a section in the packing that has been positioned.
pub struct Rect {
    /// The x position of the upper-left corner of the rectangle.
    pub x: usize,
    /// The y position of the upper-left corner of the rectangle.
    pub y: usize,
    /// The width of the rectangle.
    pub width: usize,
    /// The height of the rectangle.
    pub height: usize,
}

/// A helper structure to represent a node in the packer.
struct Node {
    /// Is this node occupied by other entries, or free to fill.
    occupied: bool,
    /// The x position of the upper-left corner of this node.
    x: usize,
    /// The y position of the upper-left corner of this node.
    y: usize,
    /// The width of this node.
    width: usize,
    /// The height of this node.
    height: usize,
    /// The node below this.
    down: Option<Rc<RefCell<Node>>>,
    /// The node right to this.
    right: Option<Rc<RefCell<Node>>>,
}

impl Node {
    /// Creates an empty node with the given upper-left corner and size.
    fn new(x: usize, y: usize, width: usize, height: usize) -> Self {
        Self{
            occupied: false,
            x, y, width, height,
            down: None,
            right: None,
        }
    }
}
