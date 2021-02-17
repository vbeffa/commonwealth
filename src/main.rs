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
        mt.data
            .resize_with(usize::pow(2, depth as u32), Default::default);
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
            // println!("{:#?}", mt.tree);
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

    // Returns a vec of size depth + 1 with proof[i] containing
    // the hash of the node at depth i that is needed for proving
    // membership of data at element index.
    //
    // The root hash at depth 0 is included so that the index into
    // the proof vector corresponds to the depth. So, for example,
    // if the tree has depth 3, then proof[2] contains the hash
    // of the node at depth 2 needed for the proof.
    //
    // TODO: memoize
    fn generate_proof(&self, index: usize) -> Vec<u64> {
        if index >= self.index {
            return Vec::new(); // error
        }

        let mut proof = Vec::with_capacity(self.depth + 1);
        proof.resize_with(self.depth + 1, Default::default);

        let mut i = index;
        // add non-root hashes
        for d in (1..self.depth + 1).rev() {
            // println!("i: {} d: {} i % 2: {}", i, d, i % 2);
            proof[d] = if i % 2 == 0 { self.tree[d][i + 1].hash } else { self.tree[d][i - 1].hash };
            // println!("proof: {:#?}", proof);
            i = i / 2;
        }

        // add root hash
        proof[0] = self.root_hash;

        proof
    }

    // fn verify(&self, index: usize) -> bool {

    // }
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
    for i in 0..8 {
        println!("proof: {:#?}", mt.generate_proof(i));
    }
}

fn calculate_hash(data: &String) -> u64 {
    let mut s = DefaultHasher::new();
    data.hash(&mut s);
    s.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merkle_depth_0() -> Result<(), String> {
        let mut mt = MerkleTree::new(0, 4506850079084802999);

        mt.add_data(&String::from("foo"));

        assert_eq!(mt.tree[0][0].hash, mt.root_hash);

        Ok(())
    }

    #[test]
    fn test_merkle_depth_1() -> Result<(), String> {
        let mut mt = MerkleTree::new(1, 17075777630381501106);

        mt.add_data(&String::from("foo"));
        mt.add_data(&String::from("bar"));

        assert_eq!(mt.tree[0][0].hash, mt.root_hash);

        Ok(())
    }

    #[test]
    fn test_merkle_proof_depth_1() -> Result<(), String> {
        let mut mt = MerkleTree::new(1, 17075777630381501106);

        mt.add_data(&String::from("foo"));
        mt.add_data(&String::from("bar"));

        assert_eq!(mt.generate_proof(0), [17075777630381501106, 3676438629107045207]);
        assert_eq!(mt.generate_proof(1), [17075777630381501106, 4506850079084802999]);

        Ok(())
    }

    #[test]
    fn test_merkle_proof_depth_2() -> Result<(), String> {
        let mut mt = MerkleTree::new(2, 4778819754073447529);

        mt.add_data(&String::from("foo"));
        mt.add_data(&String::from("bar"));
        mt.add_data(&String::from("baz"));
        mt.add_data(&String::from("yup"));

        assert_eq!(mt.generate_proof(0), [4778819754073447529, 9268692565628018440, 3676438629107045207]);
        assert_eq!(mt.generate_proof(1), [4778819754073447529, 9268692565628018440, 4506850079084802999]);
        assert_eq!(mt.generate_proof(2), [4778819754073447529, 17075777630381501106, 1968634300370677998]);
        assert_eq!(mt.generate_proof(3), [4778819754073447529, 17075777630381501106, 16260972211344176173]);

        Ok(())
    }

    #[test]
    fn test_merkle_proof_depth_3() -> Result<(), String> {
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

        assert_eq!(mt.generate_proof(0), [1556255166675498662, 1292560851973962312, 9268692565628018440, 3676438629107045207]);
        assert_eq!(mt.generate_proof(1), [1556255166675498662, 1292560851973962312, 9268692565628018440, 4506850079084802999]);
        assert_eq!(mt.generate_proof(2), [1556255166675498662, 1292560851973962312, 17075777630381501106, 1968634300370677998]);
        assert_eq!(mt.generate_proof(3), [1556255166675498662, 1292560851973962312, 17075777630381501106, 16260972211344176173]);
        assert_eq!(mt.generate_proof(4), [1556255166675498662, 4778819754073447529, 6756623144268557643, 14416090190412621920]);
        assert_eq!(mt.generate_proof(5), [1556255166675498662, 4778819754073447529, 6756623144268557643, 5587210449854392903]);
        assert_eq!(mt.generate_proof(6), [1556255166675498662, 4778819754073447529, 10865386958110225586, 9147698590242891024]);
        assert_eq!(mt.generate_proof(7), [1556255166675498662, 4778819754073447529, 10865386958110225586, 10714775279812270610]);

        Ok(())
    }
}
