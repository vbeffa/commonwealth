use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

trait Hashable {
    fn hash(&mut self) -> Option<u64>;
}

struct Node {
    left: Box<dyn Hashable>,
    right: Box<dyn Hashable>,
    // memoize hash
    hash: Option<u64>,
}

struct Leaf {
    data: String,
    // memoize hash
    hash: Option<u64>,
}

impl Hashable for Node {
    fn hash(&mut self) -> Option<u64> {
        match self.hash {
            None => match (self.left.hash(), self.right.hash()) {
                (Some(h1), Some(h2)) => self.hash = Some(calculate_hash(&format!("{}{}", h1, h2))),
                _ => ()
            },
            _ => ()
            // None => self.hash = Some(calculate_hash(&format!("{}{}", &self.left.hash(), &self.right.hash()))),
        }
        self.hash
    }
}

impl Hashable for Leaf {
    fn hash(&mut self) -> Option<u64> {
        match self.hash {
            None => self.hash = Some(calculate_hash(&self.data)),
            _ => ()
        }
        self.hash
    }
}

fn main() {
    let mut l1 = Leaf {
        data: String::from("foo"),
        hash: None,
    };
    let mut l2 = Leaf {
        data: String::from("bar"),
        hash: None,
    };
    let mut l3 = Leaf {
        data: String::from("fiz"),
        hash: None,
    };
    let mut l4 = Leaf {
        data: String::from("baz"),
        hash: None,
    };

    println!("{} {} {} {}", l1.hash().unwrap(), l2.hash().unwrap(), l3.hash().unwrap(), l4.hash().unwrap());

    let mut hash0 = Node {
        left: Box::new(l1),
        right: Box::new(l2),
        hash: None,
    };

    let mut hash1 = Node {
        left: Box::new(l3),
        right: Box::new(l4),
        hash: None,
    };

    println!("{} {}", hash0.hash().unwrap(), hash1.hash().unwrap());

    let mut root_hash = Node {
        left: Box::new(hash0),
        right: Box::new(hash1),
        hash: None,
    };

    println!("{}", root_hash.hash().unwrap())

    // let hash0_0 = Node {
    //     left: l1,
    //     right: l2,
    //     hash: hash(&l1.data),
    // };
    // let hash0_1 = hash(&l2);
    // let hash1_0 = hash(&l3);
    // let hash1_1 = hash(&l4);

    // let hash0 = hash(&format!("{}{}", hash0_0, hash0_1));
    // let hash1 = hash(&format!("{}{}", hash1_0, hash1_1));

    // println!("{}", hash0);
    // println!("{}", hash1);

    // let root_hash = hash(&format!("{}{}", hash0, hash1));

    // println!("{}", root_hash);
    // println!("{}", root_hash);
}

fn calculate_hash(data: &String) -> u64 {
    let mut s = DefaultHasher::new();
    data.hash(&mut s);
    s.finish()
}
