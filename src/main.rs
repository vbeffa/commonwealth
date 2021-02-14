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

#[derive(Default, Debug)]
struct Node {
    hash: u64,
}

#[derive(Default, Debug)]
struct Leaf {
    data: String,
}

fn new_tree(depth: usize, root_hash: u64) -> MerkleTree {
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

    // // build leaf nodes with zero data values
    // for i in 0..(2 << depth) {
    //     mt.tree[depth][i].hash = calculate_hash(&mt.data[i].data);
    // }

    // println!("{:#?}", mt);

    // build intermediate nodes up to root
    for d in (0..depth).rev() {
        mt.tree[d].resize_with(usize::pow(2, d as u32), Default::default);
        // let mut level = Vec::<Node>::new();
        // level.resize_with(2 << d, Default::default);
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

fn add_data(mt: &mut MerkleTree, data: &String) {
    // println!("adding {}", data);
    if mt.index == usize::pow(2, mt.depth as u32) {
        return; // error
    }
    mt.data[mt.index].data = data.to_string();
    mt.tree[mt.depth][mt.index].hash = calculate_hash(&data);

    let mut i = mt.index;
    let mut d = mt.depth;
    while i % 2 == 1 {
        i = i / 2;
        d = d - 1;
        mt.tree[d][i] = Node {
            hash: calculate_hash(&format!(
                "{}{}",
                mt.tree[d + 1][2 * i].hash,
                mt.tree[d + 1][2 * i + 1].hash
            )),
        };
    }

    mt.index = mt.index + 1;
}

fn main() {
    // println!("{} {}", 2 << 0, 2 << 1);
    let mut mt = new_tree(3, 1556255166675498662);

    add_data(&mut mt, &String::from("foo"));
    add_data(&mut mt, &String::from("bar"));
    add_data(&mut mt, &String::from("baz"));
    add_data(&mut mt, &String::from("yup"));
    add_data(&mut mt, &String::from("maw"));
    add_data(&mut mt, &String::from("wap"));
    add_data(&mut mt, &String::from("pit"));
    add_data(&mut mt, &String::from("fos"));

    println!("{:#?}", mt);
}

fn calculate_hash(data: &String) -> u64 {
    let mut s = DefaultHasher::new();
    data.hash(&mut s);
    s.finish()
}
