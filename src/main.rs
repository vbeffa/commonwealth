use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Debug)]
struct MerkleTree {
    depth: usize,
    root_hash: u64,
    data: Vec<Leaf>,
    tree: Vec<Vec<Node>>,
    index: usize,
}

impl MerkleTree {
    fn new(depth: usize, root_hash: u64) -> MerkleTree {
        let mut mt = MerkleTree {
            depth: depth,
            root_hash: root_hash,
            data: Vec::with_capacity(usize::pow(2, depth as u32)),
            tree: Vec::with_capacity(depth + 1),
            index: 0,
        };
    
        // initialize data with zero values
        mt.data.resize_with(usize::pow(2, depth as u32), Default::default);
        // allocate space for tree
        mt.tree.resize_with(depth + 1, Default::default);
        // initialize leaf hash with zero values
        mt.tree[depth].resize_with(usize::pow(2, depth as u32), || Node {
            hash: calculate_hash(&String::from("")),
        });
    
        // println!("{:#?}", mt);
    
        // build intermediate nodes up to root
        for d in (0..depth).rev() {
            mt.tree[d].resize_with(usize::pow(2, d as u32), Default::default);
            // println!("d: {}, 2 << (d - 1): {}", d, 2 << (d - 1));
            for i in 0..(usize::pow(2, d as u32)) {
                // println!("d = {} i = {}", d, i);
                mt.tree[d][i] = Node {
                    hash: calculate_hash(&format!(
                        "{}{}",
                        mt.tree[d + 1][2 * i].hash,
                        mt.tree[d + 1][2 * i + 1].hash
                    )),
                };
                // println!("{:#?}", mt.tree[d]);
            }
            // mt.tree.push(level);
            println!("{:#?}", mt.tree);
        }
    
        mt        
    }

    fn add_data(&mut self, data: &String) {
        if self.index == usize::pow(2, self.depth as u32) {
            return; // error
        }
        self.data[self.index].data = data.to_string();
        self.tree[self.depth][self.index].hash = calculate_hash(&data);
    
        let mut i = self.index;
        let mut d = self.depth;
        while i % 2 == 1 {
            i = i / 2;
            d = d - 1;
            self.tree[d][i] = Node {
                hash: calculate_hash(&format!(
                    "{}{}",
                    self.tree[d + 1][2 * i].hash,
                    self.tree[d + 1][2 * i + 1].hash
                )),
            };
        }
    
        self.index = self.index + 1;
    }
}

#[derive(Default, Debug)]
struct Node {
    hash: u64,
}

#[derive(Default, Debug)]
struct Leaf {
    data: String,
}

fn main() {
    // println!("{} {}", 2 << 0, 2 << 1);
    let mut mt = MerkleTree::new(3, 1556255166675498662);

    mt.add_data(&String::from("foo"));
    mt.add_data(&String::from("bar"));
    mt.add_data(&String::from("baz"));
    mt.add_data(&String::from("yup"));
    mt.add_data(&String::from("maw"));
    mt.add_data(&String::from("wap"));
    mt.add_data(&String::from("pit"));
    mt.add_data(&String::from("fos"));

    println!("{:#?}", mt);
}

fn calculate_hash(data: &String) -> u64 {
    let mut s = DefaultHasher::new();
    data.hash(&mut s);
    s.finish()
}
