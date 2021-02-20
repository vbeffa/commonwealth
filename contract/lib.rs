#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract(dynamic_storage_allocator = true)]
mod merkle {
    use ink_env::hash::{HashOutput, Sha2x256};
    use ink_storage::{Box, Vec};

    /// Defines the storage of a merkle contract.
    #[ink(storage)]
    pub struct MerkleTree {
        depth: u32,
        root_hash: Hash,
        data: Vec<Balance>,
        tree: Vec<Box<Vec<Hash>>>,
        index: u32,
    }

    impl MerkleTree {
        /// Initializes the merkle tree.
        #[ink(constructor)]
        pub fn new(depth: u32, root_hash: Hash) -> Self {
            let mut mt = Self {
                depth: depth,
                root_hash: root_hash,
                data: Vec::new(),
                tree: Vec::new(),
                index: 0,
            };

            // // push root node
            let mut root_row = Vec::new();
            root_row.push(calculate_hash(Balance::from(0u128)));
            let b = Box::new(root_row);
            mt.tree.push(b);

            // println!("{:#?}", mt);

            // build intermediate nodes up to root
            for d in (0..depth).rev() {
                let mut t_row = Vec::new();
                for i in 0..(u32::pow(2, d)) {
                    // println!("d = {} i = {}", d, i);
                    t_row.push(concat_hash(
                        &mt.tree[d + 1][2 * i],
                        &mt.tree[d + 1][2 * i + 1],
                    ));
                    // println!("{:#?}", mt.tree[d]);
                }
                // println!("{:#?}", mt.tree);
                mt.tree.push(Box::new(t_row));
            }

            mt
        }

        /// Adds an element to the tree. Elements are added sequentially.
        #[ink(message)]
        pub fn add_data(&mut self, data: Balance) {
            if self.index == u32::pow(2, self.depth as u32) {
                return; // error
            }
            self.data.push(data);
            self.tree[self.depth][self.index] = calculate_hash(data);

            let mut i = self.index;
            let mut d = self.depth;
            while i % 2 == 1 {
                i = i / 2;
                d = d - 1;
                self.tree[d][i] =
                    concat_hash(&self.tree[d + 1][2 * i], &self.tree[d + 1][2 * i + 1]);
            }

            self.index = self.index + 1;
        }

        /// Verifies that the data at position index is in the tree.
        #[ink(message)]
        pub fn verify(&self, data: Balance, index: u32) -> bool {
            let proof = self.generate_proof(index);
            let mut hash = calculate_hash(data);

            for d in (1..self.depth + 1).rev() {
                if proof[d].1 {
                    hash = concat_hash(&hash, &proof[d].0);
                } else {
                    hash = concat_hash(&proof[d].0, &hash);
                }
            }

            hash == proof[0].0
        }

        /// Returns a vec of size depth + 1 with proof[i] containing
        /// a tuple with the hash of the node at depth i and a boolean
        /// indicating whether the hash is left (false) or right (true)
        /// that is needed for proving membership of data at element index.
        ///
        /// The root hash at depth 0 is included so that the index into
        /// the proof vector corresponds to the depth. So, for example,
        /// if the tree has depth 3, then proof[2] contains the hash
        /// of the node at depth 2 needed for the proof.
        ///
        /// TODO: memoize
        fn generate_proof(&self, index: u32) -> Vec<(Hash, bool)> {
            if index >= self.index {
                return Vec::new(); // error
            }

            let mut proof = Vec::new();

            // add root hash
            proof.push((self.root_hash, true));

            let mut i = index;
            // add non-root hashes
            for d in (1..self.depth + 1).rev() {
                // println!("i: {} d: {} i % 2: {}", i, d, i % 2);
                let elem = if i % 2 == 0 {
                    (self.tree[d][i + 1], true)
                } else {
                    (self.tree[d][i - 1], false)
                };
                proof.push(elem);
                // println!("proof: {:#?}", proof);
                i = i / 2;
            }

            proof
        }
    }

    // Helper to calculate a hash value.
    fn calculate_hash(data: Balance) -> Hash {
        let mut output = <Sha2x256 as HashOutput>::Type::default(); // 256-bit buffer
        ink_env::hash_encoded::<Sha2x256, _>(&data, &mut output);
        Hash::from(output)
    }

    // Helper to concatenate two hashes.
    fn concat_hash(h1: &Hash, h2: &Hash) -> Hash {
        let mut output = <Sha2x256 as HashOutput>::Type::default(); // 256-bit buffer
        ink_env::hash_encoded::<Sha2x256, _>(h1, &mut output);
        ink_env::hash_encoded::<Sha2x256, _>(h2, &mut output);
        Hash::from(output)
    }

    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        #[test]
        fn test_merkle_depth_0() -> Result<(), String> {
            let mut mt = MerkleTree::new(
                0,
                Hash::from([
                    58, 234, 225, 192, 108, 62, 238, 181, 193, 43, 0, 221, 254, 40, 233, 54, 206,
                    236, 166, 231, 53, 178, 117, 145, 95, 227, 56, 30, 28, 157, 239, 79,
                ]),
            );

            mt.add_data(Balance::from(10u128));

            assert_eq!(mt.tree[0][0], mt.root_hash);

            Ok(())
        }

        // #[test]
        // fn test_merkle_depth_1() -> Result<(), String> {
        //     let mut mt = MerkleTree::new(
        //         1,
        //         Hash::from([
        //             64, 13, 167, 79, 119, 176, 55, 201, 53, 122, 13, 245, 64, 121, 215, 242, 35,
        //             178, 62, 214, 111, 112, 79, 0, 63, 171, 165, 102, 122, 221, 153, 65,
        //         ]),
        //     );

        //     mt.add_data(&String::from("foo"));
        //     mt.add_data(&String::from("bar"));

        //     assert_eq!(mt.tree[0][0], mt.root_hash);

        //     Ok(())
        // }

        // #[test]
        // fn test_merkle_depth_2() -> Result<(), String> {
        //     let mut mt = MerkleTree::new(
        //         2,
        //         Hash::from([
        //             254, 193, 153, 90, 96, 71, 91, 12, 93, 104, 47, 6, 229, 251, 40, 55, 59, 119,
        //             125, 90, 25, 131, 131, 98, 183, 113, 85, 28, 187, 165, 245, 83,
        //         ]),
        //     );

        //     mt.add_data(&String::from("foo"));
        //     mt.add_data(&String::from("bar"));
        //     mt.add_data(&String::from("baz"));
        //     mt.add_data(&String::from("yup"));

        //     assert_eq!(mt.tree[0][0], mt.root_hash);

        //     Ok(())
        // }

        // #[test]
        // fn test_merkle_depth_3() -> Result<(), String> {
        //     let mut mt = MerkleTree::new(
        //         3,
        //         Hash::from([
        //             112, 27, 168, 163, 68, 164, 18, 124, 211, 124, 119, 75, 52, 220, 148, 21, 181,
        //             190, 193, 125, 109, 13, 250, 251, 234, 62, 56, 9, 195, 6, 117, 124,
        //         ]),
        //     );

        //     mt.add_data(&String::from("foo"));
        //     mt.add_data(&String::from("bar"));
        //     mt.add_data(&String::from("baz"));
        //     mt.add_data(&String::from("yup"));
        //     mt.add_data(&String::from("maw"));
        //     mt.add_data(&String::from("wap"));
        //     mt.add_data(&String::from("pit"));
        //     mt.add_data(&String::from("fos"));

        //     assert_eq!(mt.tree[0][0], mt.root_hash);

        //     Ok(())
        // }

        // #[test]
        // fn test_merkle_proof_depth_1() -> Result<(), String> {
        //     let mut mt = MerkleTree::new(
        //         1,
        //         Hash::from([
        //             64, 13, 167, 79, 119, 176, 55, 201, 53, 122, 13, 245, 64, 121, 215, 242, 35,
        //             178, 62, 214, 111, 112, 79, 0, 63, 171, 165, 102, 122, 221, 153, 65,
        //         ]),
        //     );

        //     mt.add_data(&String::from("foo"));
        //     mt.add_data(&String::from("bar"));

        //     assert_eq!(
        //         mt.generate_proof(0),
        //         [
        //             (
        //                 Hash::from([
        //                     64, 13, 167, 79, 119, 176, 55, 201, 53, 122, 13, 245, 64, 121, 215,
        //                     242, 35, 178, 62, 214, 111, 112, 79, 0, 63, 171, 165, 102, 122, 221,
        //                     153, 65
        //                 ]),
        //                 true
        //             ),
        //             (
        //                 Hash::from([
        //                     63, 100, 101, 96, 218, 178, 202, 69, 181, 2, 212, 237, 74, 91, 109,
        //                     209, 179, 244, 254, 134, 222, 147, 100, 233, 81, 164, 1, 97, 63, 215,
        //                     21, 76
        //                 ]),
        //                 true
        //             )
        //         ]
        //     );
        //     assert_eq!(
        //         mt.generate_proof(1),
        //         [
        //             (
        //                 Hash::from([
        //                     64, 13, 167, 79, 119, 176, 55, 201, 53, 122, 13, 245, 64, 121, 215,
        //                     242, 35, 178, 62, 214, 111, 112, 79, 0, 63, 171, 165, 102, 122, 221,
        //                     153, 65
        //                 ]),
        //                 true
        //             ),
        //             (
        //                 Hash::from([
        //                     118, 180, 157, 7, 191, 196, 18, 209, 75, 222, 7, 179, 69, 87, 179, 113,
        //                     14, 3, 112, 123, 66, 146, 83, 250, 146, 146, 122, 28, 178, 156, 155,
        //                     124
        //                 ]),
        //                 false
        //             )
        //         ]
        //     );

        //     Ok(())
        // }

        // #[test]
        // fn test_merkle_proof_depth_2() -> Result<(), String> {
        //     let mut mt = MerkleTree::new(
        //         2,
        //         Hash::from([
        //             254, 193, 153, 90, 96, 71, 91, 12, 93, 104, 47, 6, 229, 251, 40, 55, 59, 119,
        //             125, 90, 25, 131, 131, 98, 183, 113, 85, 28, 187, 165, 245, 83,
        //         ]),
        //     );

        //     mt.add_data(&String::from("foo"));
        //     mt.add_data(&String::from("bar"));
        //     mt.add_data(&String::from("baz"));
        //     mt.add_data(&String::from("yup"));

        //     assert_eq!(
        //         mt.generate_proof(0),
        //         [
        //             (
        //                 Hash::from([
        //                     254, 193, 153, 90, 96, 71, 91, 12, 93, 104, 47, 6, 229, 251, 40, 55,
        //                     59, 119, 125, 90, 25, 131, 131, 98, 183, 113, 85, 28, 187, 165, 245,
        //                     83
        //                 ]),
        //                 true
        //             ),
        //             (
        //                 Hash::from([
        //                     135, 79, 28, 146, 14, 207, 62, 2, 4, 90, 44, 192, 10, 131, 2, 170, 117,
        //                     50, 86, 185, 145, 144, 75, 5, 179, 238, 212, 167, 40, 71, 190, 158
        //                 ]),
        //                 true
        //             ),
        //             (
        //                 Hash::from([
        //                     63, 100, 101, 96, 218, 178, 202, 69, 181, 2, 212, 237, 74, 91, 109,
        //                     209, 179, 244, 254, 134, 222, 147, 100, 233, 81, 164, 1, 97, 63, 215,
        //                     21, 76
        //                 ]),
        //                 true
        //             )
        //         ]
        //     );
        //     assert_eq!(
        //         mt.generate_proof(1),
        //         [
        //             (
        //                 Hash::from([
        //                     254, 193, 153, 90, 96, 71, 91, 12, 93, 104, 47, 6, 229, 251, 40, 55,
        //                     59, 119, 125, 90, 25, 131, 131, 98, 183, 113, 85, 28, 187, 165, 245,
        //                     83
        //                 ]),
        //                 true
        //             ),
        //             (
        //                 Hash::from([
        //                     135, 79, 28, 146, 14, 207, 62, 2, 4, 90, 44, 192, 10, 131, 2, 170, 117,
        //                     50, 86, 185, 145, 144, 75, 5, 179, 238, 212, 167, 40, 71, 190, 158
        //                 ]),
        //                 true
        //             ),
        //             (
        //                 Hash::from([
        //                     118, 180, 157, 7, 191, 196, 18, 209, 75, 222, 7, 179, 69, 87, 179, 113,
        //                     14, 3, 112, 123, 66, 146, 83, 250, 146, 146, 122, 28, 178, 156, 155,
        //                     124
        //                 ]),
        //                 false
        //             )
        //         ]
        //     );
        //     assert_eq!(
        //         mt.generate_proof(2),
        //         [
        //             (
        //                 Hash::from([
        //                     254, 193, 153, 90, 96, 71, 91, 12, 93, 104, 47, 6, 229, 251, 40, 55,
        //                     59, 119, 125, 90, 25, 131, 131, 98, 183, 113, 85, 28, 187, 165, 245,
        //                     83
        //                 ]),
        //                 true
        //             ),
        //             (
        //                 Hash::from([
        //                     64, 13, 167, 79, 119, 176, 55, 201, 53, 122, 13, 245, 64, 121, 215,
        //                     242, 35, 178, 62, 214, 111, 112, 79, 0, 63, 171, 165, 102, 122, 221,
        //                     153, 65
        //                 ]),
        //                 false
        //             ),
        //             (
        //                 Hash::from([
        //                     133, 8, 186, 149, 254, 122, 181, 59, 180, 27, 104, 39, 157, 47, 94,
        //                     220, 218, 45, 213, 77, 175, 23, 162, 14, 166, 244, 7, 220, 22, 40, 192,
        //                     124
        //                 ]),
        //                 true
        //             )
        //         ]
        //     );
        //     assert_eq!(
        //         mt.generate_proof(3),
        //         [
        //             (
        //                 Hash::from([
        //                     254, 193, 153, 90, 96, 71, 91, 12, 93, 104, 47, 6, 229, 251, 40, 55,
        //                     59, 119, 125, 90, 25, 131, 131, 98, 183, 113, 85, 28, 187, 165, 245,
        //                     83
        //                 ]),
        //                 true
        //             ),
        //             (
        //                 Hash::from([
        //                     64, 13, 167, 79, 119, 176, 55, 201, 53, 122, 13, 245, 64, 121, 215,
        //                     242, 35, 178, 62, 214, 111, 112, 79, 0, 63, 171, 165, 102, 122, 221,
        //                     153, 65
        //                 ]),
        //                 false
        //             ),
        //             (
        //                 Hash::from([
        //                     155, 81, 107, 153, 57, 165, 157, 11, 205, 73, 45, 210, 180, 254, 24,
        //                     181, 135, 101, 92, 17, 108, 170, 185, 170, 117, 148, 188, 30, 20, 34,
        //                     16, 125
        //                 ]),
        //                 false
        //             )
        //         ]
        //     );

        //     Ok(())
        // }

        // #[test]
        // fn test_merkle_proof_depth_3() -> Result<(), String> {
        //     let mut mt = MerkleTree::new(
        //         3,
        //         Hash::from([
        //             112, 27, 168, 163, 68, 164, 18, 124, 211, 124, 119, 75, 52, 220, 148, 21, 181,
        //             190, 193, 125, 109, 13, 250, 251, 234, 62, 56, 9, 195, 6, 117, 124,
        //         ]),
        //     );

        //     mt.add_data(&String::from("foo"));
        //     mt.add_data(&String::from("bar"));
        //     mt.add_data(&String::from("baz"));
        //     mt.add_data(&String::from("yup"));
        //     mt.add_data(&String::from("maw"));
        //     mt.add_data(&String::from("wap"));
        //     mt.add_data(&String::from("pit"));
        //     mt.add_data(&String::from("fos"));

        //     println!("{:#?}", mt);

        //     assert_eq!(
        //         mt.generate_proof(0),
        //         [
        //             (
        //                 Hash::from([
        //                     112, 27, 168, 163, 68, 164, 18, 124, 211, 124, 119, 75, 52, 220, 148,
        //                     21, 181, 190, 193, 125, 109, 13, 250, 251, 234, 62, 56, 9, 195, 6, 117,
        //                     124
        //                 ]),
        //                 true
        //             ),
        //             (
        //                 Hash::from([
        //                     229, 68, 83, 153, 144, 156, 20, 222, 214, 217, 113, 124, 47, 76, 134,
        //                     95, 112, 56, 117, 26, 215, 229, 22, 196, 206, 103, 219, 253, 26, 221,
        //                     242, 230
        //                 ]),
        //                 true
        //             ),
        //             (
        //                 Hash::from([
        //                     135, 79, 28, 146, 14, 207, 62, 2, 4, 90, 44, 192, 10, 131, 2, 170, 117,
        //                     50, 86, 185, 145, 144, 75, 5, 179, 238, 212, 167, 40, 71, 190, 158
        //                 ]),
        //                 true
        //             ),
        //             (
        //                 Hash::from([
        //                     63, 100, 101, 96, 218, 178, 202, 69, 181, 2, 212, 237, 74, 91, 109,
        //                     209, 179, 244, 254, 134, 222, 147, 100, 233, 81, 164, 1, 97, 63, 215,
        //                     21, 76
        //                 ]),
        //                 true
        //             )
        //         ]
        //     );
        //     assert_eq!(
        //         mt.generate_proof(1),
        //         [
        //             (
        //                 Hash::from([
        //                     112, 27, 168, 163, 68, 164, 18, 124, 211, 124, 119, 75, 52, 220, 148,
        //                     21, 181, 190, 193, 125, 109, 13, 250, 251, 234, 62, 56, 9, 195, 6, 117,
        //                     124
        //                 ]),
        //                 true
        //             ),
        //             (
        //                 Hash::from([
        //                     229, 68, 83, 153, 144, 156, 20, 222, 214, 217, 113, 124, 47, 76, 134,
        //                     95, 112, 56, 117, 26, 215, 229, 22, 196, 206, 103, 219, 253, 26, 221,
        //                     242, 230
        //                 ]),
        //                 true
        //             ),
        //             (
        //                 Hash::from([
        //                     135, 79, 28, 146, 14, 207, 62, 2, 4, 90, 44, 192, 10, 131, 2, 170, 117,
        //                     50, 86, 185, 145, 144, 75, 5, 179, 238, 212, 167, 40, 71, 190, 158
        //                 ]),
        //                 true
        //             ),
        //             (
        //                 Hash::from([
        //                     118, 180, 157, 7, 191, 196, 18, 209, 75, 222, 7, 179, 69, 87, 179, 113,
        //                     14, 3, 112, 123, 66, 146, 83, 250, 146, 146, 122, 28, 178, 156, 155,
        //                     124
        //                 ]),
        //                 false
        //             )
        //         ]
        //     );
        //     assert_eq!(
        //         mt.generate_proof(2),
        //         [
        //             (
        //                 Hash::from([
        //                     112, 27, 168, 163, 68, 164, 18, 124, 211, 124, 119, 75, 52, 220, 148,
        //                     21, 181, 190, 193, 125, 109, 13, 250, 251, 234, 62, 56, 9, 195, 6, 117,
        //                     124
        //                 ]),
        //                 true
        //             ),
        //             (
        //                 Hash::from([
        //                     229, 68, 83, 153, 144, 156, 20, 222, 214, 217, 113, 124, 47, 76, 134,
        //                     95, 112, 56, 117, 26, 215, 229, 22, 196, 206, 103, 219, 253, 26, 221,
        //                     242, 230
        //                 ]),
        //                 true
        //             ),
        //             (
        //                 Hash::from([
        //                     64, 13, 167, 79, 119, 176, 55, 201, 53, 122, 13, 245, 64, 121, 215,
        //                     242, 35, 178, 62, 214, 111, 112, 79, 0, 63, 171, 165, 102, 122, 221,
        //                     153, 65
        //                 ]),
        //                 false
        //             ),
        //             (
        //                 Hash::from([
        //                     133, 8, 186, 149, 254, 122, 181, 59, 180, 27, 104, 39, 157, 47, 94,
        //                     220, 218, 45, 213, 77, 175, 23, 162, 14, 166, 244, 7, 220, 22, 40, 192,
        //                     124
        //                 ]),
        //                 true
        //             )
        //         ]
        //     );
        //     assert_eq!(
        //         mt.generate_proof(3),
        //         [
        //             (
        //                 Hash::from([
        //                     112, 27, 168, 163, 68, 164, 18, 124, 211, 124, 119, 75, 52, 220, 148,
        //                     21, 181, 190, 193, 125, 109, 13, 250, 251, 234, 62, 56, 9, 195, 6, 117,
        //                     124
        //                 ]),
        //                 true
        //             ),
        //             (
        //                 Hash::from([
        //                     229, 68, 83, 153, 144, 156, 20, 222, 214, 217, 113, 124, 47, 76, 134,
        //                     95, 112, 56, 117, 26, 215, 229, 22, 196, 206, 103, 219, 253, 26, 221,
        //                     242, 230
        //                 ]),
        //                 true
        //             ),
        //             (
        //                 Hash::from([
        //                     64, 13, 167, 79, 119, 176, 55, 201, 53, 122, 13, 245, 64, 121, 215,
        //                     242, 35, 178, 62, 214, 111, 112, 79, 0, 63, 171, 165, 102, 122, 221,
        //                     153, 65
        //                 ]),
        //                 false
        //             ),
        //             (
        //                 Hash::from([
        //                     155, 81, 107, 153, 57, 165, 157, 11, 205, 73, 45, 210, 180, 254, 24,
        //                     181, 135, 101, 92, 17, 108, 170, 185, 170, 117, 148, 188, 30, 20, 34,
        //                     16, 125
        //                 ]),
        //                 false
        //             )
        //         ]
        //     );
        //     assert_eq!(
        //         mt.generate_proof(4),
        //         [
        //             (
        //                 Hash::from([
        //                     112, 27, 168, 163, 68, 164, 18, 124, 211, 124, 119, 75, 52, 220, 148,
        //                     21, 181, 190, 193, 125, 109, 13, 250, 251, 234, 62, 56, 9, 195, 6, 117,
        //                     124
        //                 ]),
        //                 true
        //             ),
        //             (
        //                 Hash::from([
        //                     254, 193, 153, 90, 96, 71, 91, 12, 93, 104, 47, 6, 229, 251, 40, 55,
        //                     59, 119, 125, 90, 25, 131, 131, 98, 183, 113, 85, 28, 187, 165, 245,
        //                     83
        //                 ]),
        //                 false
        //             ),
        //             (
        //                 Hash::from([
        //                     140, 206, 71, 81, 207, 138, 152, 233, 136, 163, 36, 217, 62, 66, 198,
        //                     33, 184, 126, 89, 6, 37, 118, 204, 199, 98, 208, 64, 247, 39, 235, 34,
        //                     174
        //                 ]),
        //                 true
        //             ),
        //             (
        //                 Hash::from([
        //                     85, 114, 5, 36, 70, 244, 220, 223, 134, 177, 233, 45, 191, 186, 137,
        //                     181, 147, 210, 213, 223, 37, 200, 175, 101, 36, 8, 228, 120, 39, 171,
        //                     103, 59
        //                 ]),
        //                 true
        //             )
        //         ]
        //     );
        //     assert_eq!(
        //         mt.generate_proof(5),
        //         [
        //             (
        //                 Hash::from([
        //                     112, 27, 168, 163, 68, 164, 18, 124, 211, 124, 119, 75, 52, 220, 148,
        //                     21, 181, 190, 193, 125, 109, 13, 250, 251, 234, 62, 56, 9, 195, 6, 117,
        //                     124
        //                 ]),
        //                 true
        //             ),
        //             (
        //                 Hash::from([
        //                     254, 193, 153, 90, 96, 71, 91, 12, 93, 104, 47, 6, 229, 251, 40, 55,
        //                     59, 119, 125, 90, 25, 131, 131, 98, 183, 113, 85, 28, 187, 165, 245,
        //                     83
        //                 ]),
        //                 false
        //             ),
        //             (
        //                 Hash::from([
        //                     140, 206, 71, 81, 207, 138, 152, 233, 136, 163, 36, 217, 62, 66, 198,
        //                     33, 184, 126, 89, 6, 37, 118, 204, 199, 98, 208, 64, 247, 39, 235, 34,
        //                     174
        //                 ]),
        //                 true
        //             ),
        //             (
        //                 Hash::from([
        //                     138, 150, 65, 116, 174, 72, 169, 234, 89, 107, 224, 179, 152, 162, 114,
        //                     189, 203, 255, 52, 203, 121, 125, 196, 40, 88, 93, 126, 252, 2, 62, 27,
        //                     196
        //                 ]),
        //                 false
        //             )
        //         ]
        //     );
        //     assert_eq!(
        //         mt.generate_proof(6),
        //         [
        //             (
        //                 Hash::from([
        //                     112, 27, 168, 163, 68, 164, 18, 124, 211, 124, 119, 75, 52, 220, 148,
        //                     21, 181, 190, 193, 125, 109, 13, 250, 251, 234, 62, 56, 9, 195, 6, 117,
        //                     124
        //                 ]),
        //                 true
        //             ),
        //             (
        //                 Hash::from([
        //                     254, 193, 153, 90, 96, 71, 91, 12, 93, 104, 47, 6, 229, 251, 40, 55,
        //                     59, 119, 125, 90, 25, 131, 131, 98, 183, 113, 85, 28, 187, 165, 245,
        //                     83
        //                 ]),
        //                 false
        //             ),
        //             (
        //                 Hash::from([
        //                     42, 146, 158, 253, 90, 152, 165, 44, 153, 192, 65, 110, 211, 164, 183,
        //                     17, 51, 156, 13, 20, 200, 59, 61, 98, 4, 177, 153, 131, 83, 147, 109,
        //                     133
        //                 ]),
        //                 false
        //             ),
        //             (
        //                 Hash::from([
        //                     150, 40, 244, 219, 122, 25, 223, 75, 239, 193, 60, 84, 103, 37, 105,
        //                     27, 217, 9, 155, 141, 174, 244, 183, 113, 17, 26, 6, 236, 10, 50, 89,
        //                     174
        //                 ]),
        //                 true
        //             )
        //         ]
        //     );
        //     assert_eq!(
        //         mt.generate_proof(7),
        //         [
        //             (
        //                 Hash::from([
        //                     112, 27, 168, 163, 68, 164, 18, 124, 211, 124, 119, 75, 52, 220, 148,
        //                     21, 181, 190, 193, 125, 109, 13, 250, 251, 234, 62, 56, 9, 195, 6, 117,
        //                     124
        //                 ]),
        //                 true
        //             ),
        //             (
        //                 Hash::from([
        //                     254, 193, 153, 90, 96, 71, 91, 12, 93, 104, 47, 6, 229, 251, 40, 55,
        //                     59, 119, 125, 90, 25, 131, 131, 98, 183, 113, 85, 28, 187, 165, 245,
        //                     83
        //                 ]),
        //                 false
        //             ),
        //             (
        //                 Hash::from([
        //                     42, 146, 158, 253, 90, 152, 165, 44, 153, 192, 65, 110, 211, 164, 183,
        //                     17, 51, 156, 13, 20, 200, 59, 61, 98, 4, 177, 153, 131, 83, 147, 109,
        //                     133
        //                 ]),
        //                 false
        //             ),
        //             (
        //                 Hash::from([
        //                     228, 188, 61, 70, 88, 133, 247, 129, 164, 203, 208, 77, 24, 254, 165,
        //                     165, 237, 16, 209, 177, 74, 104, 151, 13, 152, 167, 151, 45, 59, 232,
        //                     134, 155
        //                 ]),
        //                 false
        //             )
        //         ]
        //     );

        //     Ok(())
        // }

        // #[test]
        // fn test_merkle_verify_depth_1() -> Result<(), String> {
        //     let mut mt = MerkleTree::new(
        //         1,
        //         Hash::from([
        //             64, 13, 167, 79, 119, 176, 55, 201, 53, 122, 13, 245, 64, 121, 215, 242, 35,
        //             178, 62, 214, 111, 112, 79, 0, 63, 171, 165, 102, 122, 221, 153, 65,
        //         ]),
        //     );

        //     mt.add_data(&String::from("foo"));
        //     mt.add_data(&String::from("bar"));

        //     assert_eq!(mt.verify(&String::from("foo"), 0), true);
        //     assert_eq!(mt.verify(&String::from("bar"), 1), true);
        //     assert_eq!(mt.verify(&String::from("bar"), 0), false);
        //     assert_eq!(mt.verify(&String::from("foo"), 1), false);

        //     Ok(())
        // }

        // #[test]
        // fn test_merkle_verify_depth_2() -> Result<(), String> {
        //     let mut mt = MerkleTree::new(
        //         2,
        //         Hash::from([
        //             254, 193, 153, 90, 96, 71, 91, 12, 93, 104, 47, 6, 229, 251, 40, 55, 59, 119,
        //             125, 90, 25, 131, 131, 98, 183, 113, 85, 28, 187, 165, 245, 83,
        //         ]),
        //     );

        //     mt.add_data(&String::from("foo"));
        //     mt.add_data(&String::from("bar"));
        //     mt.add_data(&String::from("baz"));
        //     mt.add_data(&String::from("yup"));

        //     assert_eq!(mt.verify(&String::from("foo"), 0), true);
        //     assert_eq!(mt.verify(&String::from("bar"), 1), true);
        //     assert_eq!(mt.verify(&String::from("baz"), 2), true);
        //     assert_eq!(mt.verify(&String::from("yup"), 3), true);
        //     assert_eq!(mt.verify(&String::from("bar"), 0), false);
        //     assert_eq!(mt.verify(&String::from("baz"), 1), false);
        //     assert_eq!(mt.verify(&String::from("yup"), 2), false);
        //     assert_eq!(mt.verify(&String::from("foo"), 3), false);

        //     Ok(())
        // }

        // #[test]
        // fn test_merkle_verify_depth_3() -> Result<(), String> {
        //     let mut mt = MerkleTree::new(
        //         3,
        //         Hash::from([
        //             112, 27, 168, 163, 68, 164, 18, 124, 211, 124, 119, 75, 52, 220, 148, 21, 181,
        //             190, 193, 125, 109, 13, 250, 251, 234, 62, 56, 9, 195, 6, 117, 124,
        //         ]),
        //     );

        //     mt.add_data(&String::from("foo"));
        //     mt.add_data(&String::from("bar"));
        //     mt.add_data(&String::from("baz"));
        //     mt.add_data(&String::from("yup"));
        //     mt.add_data(&String::from("maw"));
        //     mt.add_data(&String::from("wap"));
        //     mt.add_data(&String::from("pit"));
        //     mt.add_data(&String::from("fos"));

        //     println!("{:#?}", mt);

        //     assert_eq!(mt.verify(&String::from("foo"), 0), true);
        //     assert_eq!(mt.verify(&String::from("bar"), 1), true);
        //     assert_eq!(mt.verify(&String::from("baz"), 2), true);
        //     assert_eq!(mt.verify(&String::from("yup"), 3), true);
        //     assert_eq!(mt.verify(&String::from("maw"), 4), true);
        //     assert_eq!(mt.verify(&String::from("wap"), 5), true);
        //     assert_eq!(mt.verify(&String::from("pit"), 6), true);
        //     assert_eq!(mt.verify(&String::from("fos"), 7), true);
        //     assert_eq!(mt.verify(&String::from("bar"), 0), false);
        //     assert_eq!(mt.verify(&String::from("baz"), 1), false);
        //     assert_eq!(mt.verify(&String::from("yup"), 2), false);
        //     assert_eq!(mt.verify(&String::from("maw"), 3), false);
        //     assert_eq!(mt.verify(&String::from("wap"), 4), false);
        //     assert_eq!(mt.verify(&String::from("pit"), 5), false);
        //     assert_eq!(mt.verify(&String::from("fos"), 6), false);
        //     assert_eq!(mt.verify(&String::from("foo"), 7), false);

        //     Ok(())
        // }
    }
}
