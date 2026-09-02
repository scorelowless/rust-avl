use rust_avl::{avl, AVLTree, PoorString};

fn checker<T>(x: Option<T>) -> T {
    match x {
        Some(v) => v,
        None => {
            eprintln!("Error encountered");
            std::process::exit(1);
        }
    }
}

fn main() {
    let mut tree1 = checker(avl!(
        1 => "one",
        2 => "two",
        3 => "three"
    ));

    let mut tree2 = AVLTree::new();
    tree2.insert(4, checker(PoorString::new(b"four")));
    tree2.insert(5, checker(PoorString::new(b"five")));
    tree2.insert(6, checker(PoorString::new(b"six")));

    if tree1.contains(2) {
        let value = tree1.get(2);
        match value {
            Some(v) => println!("Key 2 maps to value: {}", checker(v.as_str())),
            None => println!("Key 2 not found"),
        }
    }
    match tree2.get(6) {
        Some(v) => println!("Key 6 maps to value: {}", checker(v.as_str())),
        None => println!("Key 6 not found"),
    }

    if tree1.delete(3) {
        println!("Key 3 deleted successfully");
    }
    tree1.free();
    tree2.free();
}