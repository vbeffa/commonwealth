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
    // a tuple with the hash of the node at depth i and a boolean
    // indicating whether the hash is left (false) or right (true)
    // that is needed for proving membership of data at element index.
    //
    // The root hash at depth 0 is included so that the index into
    // the proof vector corresponds to the depth. So, for example,
    // if the tree has depth 3, then proof[2] contains the hash
    // of the node at depth 2 needed for the proof.
    //
    // TODO: memoize
    fn generate_proof(&self, index: usize) -> Vec<(u64, bool)> {
        if index >= self.index {
            return Vec::new(); // error
        }

        let mut proof = Vec::with_capacity(self.depth + 1);
        proof.resize_with(self.depth + 1, Default::default);

        let mut i = index;
        // add non-root hashes
        for d in (1..self.depth + 1).rev() {
            // println!("i: {} d: {} i % 2: {}", i, d, i % 2);
            proof[d] = if i % 2 == 0 {
                (self.tree[d][i + 1].hash, true)
            } else {
                (self.tree[d][i - 1].hash, false)
            };
            // println!("proof: {:#?}", proof);
            i = i / 2;
        }

        // add root hash
        proof[0] = (self.root_hash, true);

        proof
    }

    fn verify(&self, data: &String, proof: &Vec<(u64, bool)>) -> bool {
        let mut hash = calculate_hash(data);

        for d in (1..self.depth + 1).rev() {
            println!("{}", hash);
            if proof[d].1 {
                hash = calculate_hash(&format!("{}{}", hash, proof[d].0));
            } else {
                hash = calculate_hash(&format!("{}{}", proof[d].0, hash));
            }
        }

        hash == proof[0].0
    }
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
        let proof = mt.generate_proof(i);
        println!(
            "proof: {:#?} verify: {}",
            proof,
            mt.verify(&mt.data[i].data, &proof)
        );
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

        assert_eq!(
            mt.generate_proof(0),
            [(17075777630381501106, true), (3676438629107045207, true)]
        );
        assert_eq!(
            mt.generate_proof(1),
            [(17075777630381501106, true), (4506850079084802999, false)]
        );

        Ok(())
    }

    #[test]
    fn test_merkle_proof_depth_2() -> Result<(), String> {
        let mut mt = MerkleTree::new(2, 4778819754073447529);

        mt.add_data(&String::from("foo"));
        mt.add_data(&String::from("bar"));
        mt.add_data(&String::from("baz"));
        mt.add_data(&String::from("yup"));

        assert_eq!(
            mt.generate_proof(0),
            [
                (4778819754073447529, true),
                (9268692565628018440, true),
                (3676438629107045207, true)
            ]
        );
        assert_eq!(
            mt.generate_proof(1),
            [
                (4778819754073447529, true),
                (9268692565628018440, true),
                (4506850079084802999, false)
            ]
        );
        assert_eq!(
            mt.generate_proof(2),
            [
                (4778819754073447529, true),
                (17075777630381501106, false),
                (1968634300370677998, true)
            ]
        );
        assert_eq!(
            mt.generate_proof(3),
            [
                (4778819754073447529, true),
                (17075777630381501106, false),
                (16260972211344176173, false)
            ]
        );

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

        assert_eq!(
            mt.generate_proof(0),
            [
                (1556255166675498662, true),
                (1292560851973962312, true),
                (9268692565628018440, true),
                (3676438629107045207, true)
            ]
        );
        assert_eq!(
            mt.generate_proof(1),
            [
                (1556255166675498662, true),
                (1292560851973962312, true),
                (9268692565628018440, true),
                (4506850079084802999, false)
            ]
        );
        assert_eq!(
            mt.generate_proof(2),
            [
                (1556255166675498662, true),
                (1292560851973962312, true),
                (17075777630381501106, false),
                (1968634300370677998, true)
            ]
        );
        assert_eq!(
            mt.generate_proof(3),
            [
                (1556255166675498662, true),
                (1292560851973962312, true),
                (17075777630381501106, false),
                (16260972211344176173, false)
            ]
        );
        assert_eq!(
            mt.generate_proof(4),
            [
                (1556255166675498662, true),
                (4778819754073447529, false),
                (6756623144268557643, true),
                (14416090190412621920, true)
            ]
        );
        assert_eq!(
            mt.generate_proof(5),
            [
                (1556255166675498662, true),
                (4778819754073447529, false),
                (6756623144268557643, true),
                (5587210449854392903, false)
            ]
        );
        assert_eq!(
            mt.generate_proof(6),
            [
                (1556255166675498662, true),
                (4778819754073447529, false),
                (10865386958110225586, false),
                (9147698590242891024, true)
            ]
        );
        assert_eq!(
            mt.generate_proof(7),
            [
                (1556255166675498662, true),
                (4778819754073447529, false),
                (10865386958110225586, false),
                (10714775279812270610, false)
            ]
        );

        Ok(())
    }

    #[test]
    fn test_merkle_verify_depth_1() -> Result<(), String> {
        let mut mt = MerkleTree::new(1, 17075777630381501106);

        mt.add_data(&String::from("foo"));
        mt.add_data(&String::from("bar"));

        assert_eq!(mt.verify(&String::from("foo"), &mt.generate_proof(0)), true);
        assert_eq!(mt.verify(&String::from("bar"), &mt.generate_proof(1)), true);
        assert_eq!(
            mt.verify(&String::from("bar"), &mt.generate_proof(0)),
            false
        );
        assert_eq!(
            mt.verify(&String::from("foo"), &mt.generate_proof(1)),
            false
        );

        Ok(())
    }

    #[test]
    fn test_merkle_verify_depth_2() -> Result<(), String> {
        let mut mt = MerkleTree::new(2, 4778819754073447529);

        mt.add_data(&String::from("foo"));
        mt.add_data(&String::from("bar"));
        mt.add_data(&String::from("baz"));
        mt.add_data(&String::from("yup"));

        assert_eq!(mt.verify(&String::from("foo"), &mt.generate_proof(0)), true);
        assert_eq!(mt.verify(&String::from("bar"), &mt.generate_proof(1)), true);
        assert_eq!(mt.verify(&String::from("baz"), &mt.generate_proof(2)), true);
        assert_eq!(mt.verify(&String::from("yup"), &mt.generate_proof(3)), true);
        assert_eq!(
            mt.verify(&String::from("bar"), &mt.generate_proof(0)),
            false
        );
        assert_eq!(
            mt.verify(&String::from("baz"), &mt.generate_proof(1)),
            false
        );
        assert_eq!(
            mt.verify(&String::from("yup"), &mt.generate_proof(2)),
            false
        );
        assert_eq!(
            mt.verify(&String::from("foo"), &mt.generate_proof(3)),
            false
        );

        Ok(())
    }

    #[test]
    fn test_merkle_verify_depth_3() -> Result<(), String> {
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

        assert_eq!(mt.verify(&String::from("foo"), &mt.generate_proof(0)), true);
        assert_eq!(mt.verify(&String::from("bar"), &mt.generate_proof(1)), true);
        assert_eq!(mt.verify(&String::from("baz"), &mt.generate_proof(2)), true);
        assert_eq!(mt.verify(&String::from("yup"), &mt.generate_proof(3)), true);
        assert_eq!(mt.verify(&String::from("maw"), &mt.generate_proof(4)), true);
        assert_eq!(mt.verify(&String::from("wap"), &mt.generate_proof(5)), true);
        assert_eq!(mt.verify(&String::from("pit"), &mt.generate_proof(6)), true);
        assert_eq!(mt.verify(&String::from("fos"), &mt.generate_proof(7)), true);
        assert_eq!(
            mt.verify(&String::from("bar"), &mt.generate_proof(0)),
            false
        );
        assert_eq!(
            mt.verify(&String::from("baz"), &mt.generate_proof(1)),
            false
        );
        assert_eq!(
            mt.verify(&String::from("yup"), &mt.generate_proof(2)),
            false
        );
        assert_eq!(
            mt.verify(&String::from("maw"), &mt.generate_proof(3)),
            false
        );
        assert_eq!(
            mt.verify(&String::from("wap"), &mt.generate_proof(4)),
            false
        );
        assert_eq!(
            mt.verify(&String::from("pit"), &mt.generate_proof(5)),
            false
        );
        assert_eq!(
            mt.verify(&String::from("fos"), &mt.generate_proof(6)),
            false
        );
        assert_eq!(
            mt.verify(&String::from("foo"), &mt.generate_proof(7)),
            false
        );

        Ok(())
    }
}
