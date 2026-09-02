use std::ffi::CStr;
use libc::{c_char};
use std::alloc::Layout;

pub struct PoorString {
    ptr: *const u8,
    len: usize,
    valid: bool,
}

impl PoorString {
    pub fn new(s: &[u8]) -> Option<Self> {
        if s.is_empty() {
            return None;
        }
        let layout = Layout::from_size_align(s.len(), 1).ok()?;
        let ptr = unsafe { std::alloc::alloc(layout) };
        if ptr.is_null() {
            return None;
        }
        unsafe { std::ptr::copy_nonoverlapping(s.as_ptr(), ptr, s.len()); }
        Some(Self {
            ptr,
            len: s.len(),
            valid: true,
        })
    }

    pub fn as_str(&self) -> Option<&str> {
        let slice = unsafe { std::slice::from_raw_parts(self.ptr, self.len) };
        std::str::from_utf8(slice).ok()
    }

    fn free(&mut self) {
        if !self.valid {
            return;
        }
        if let Ok(layout) = Layout::from_size_align(self.len, 1) {
            unsafe { std::alloc::dealloc(self.ptr as *mut u8, layout); }
            self.valid = false;
        } // in case of Err we do nothing
    }
}

struct Node {
    key: u64,
    value: PoorString,
    left: *mut Node,
    right: *mut Node,
    height: i32
}

impl Node {
    unsafe fn new(key: u64, value: PoorString) -> Option<*mut Self> {
        let layout = Layout::new::<Self>();
        let ptr = unsafe { std::alloc::alloc(layout) } as *mut Self;
        if ptr.is_null() {
            return None;
        }
        unsafe {
            std::ptr::write(ptr, Self {
                key,
                value,
                left: std::ptr::null_mut(),
                right: std::ptr::null_mut(),
                height: 1
            });
        }
        Some(ptr)
    }

    fn free(node: *mut Self) {
        if node.is_null() {
            return;
        }
        unsafe {
            (*node).value.free();
            let layout = Layout::new::<Self>();
            std::alloc::dealloc(node as *mut u8, layout);
        }
    }

    unsafe fn height(node: *mut Self) -> i32 {
        if node.is_null() {
            0
        } else {
            unsafe { (*node).height }
        }
    }

    unsafe fn balance(node: *mut Self) -> i32 {
        if node.is_null() {
            return 0;
        }
        unsafe {
            let left_height = Self::height((*node).left);
            let right_height = Self::height((*node).right);
            left_height - right_height
        }
    }

    unsafe fn update_height(&mut self) {
        unsafe {
            self.height = 1 + std::cmp::max(Self::height(self.left), Self::height(self.right));
        }
    }

    unsafe fn move_key_value_from_to(from: *mut Self, to: *mut Self) {
        if from.is_null() || to.is_null() {
            return;
        }
        unsafe {
            (*to).key = (*from).key;
            (*to).value.free();
            (*to).value = PoorString {
                ptr: (*from).value.ptr,
                len: (*from).value.len,
                valid: (*from).value.valid,
            };
            (*from).value.valid = false;
        }

    }
}

#[repr(C)]
pub struct AVLTree {
    root: *mut Node,
}

impl Default for AVLTree {
    fn default() -> Self {
        Self::new()
    }
}

impl AVLTree {
    pub fn new() -> Self {
        Self { root: std::ptr::null_mut() }
    }

    fn free_node(&self, node: *mut Node) {
        if node.is_null() {
            return;
        }
        unsafe {
            self.free_node((*node).left);
            self.free_node((*node).right);
            Node::free(node);
        }
    }

    pub fn free(&self) {
        self.free_node(self.root);
    }

    unsafe fn rotate_right(node: *mut Node) -> *mut Node {
        unsafe {
            let l = (*node).left;
            let lr = (*l).right;

            (*l).right = node;
            (*node).left = lr;

            (*node).update_height();
            (*l).update_height();

            l
        }
    }

    unsafe fn rotate_left(node: *mut Node) -> *mut Node {
        unsafe {
            let r = (*node).right;
            let rl = (*r).left;

            (*r).left = node;
            (*node).right = rl;

            (*node).update_height();
            (*r).update_height();

            r
        }
    }

    unsafe fn rebalance(node: *mut Node) -> Option<*mut Node> {
        unsafe {
            let balance = Node::balance(node);

            if balance > 1 {
                if Node::balance((*node).left) < 0 {
                    (*node).left = Self::rotate_left((*node).left); // LR
                }
                return Some(Self::rotate_right(node)); // LL
            }
            if balance < -1 {
                if Node::balance((*node).right) > 0 {
                    (*node).right = Self::rotate_right((*node).right); // RL
                }
                return Some(Self::rotate_left(node)); // RR
            }
            Some(node)
        }
    }

    // error in this function means that the key already exists or a memory allocation error occurred
    unsafe fn insert_node(&self, node: *mut Node, key: u64, value: PoorString) -> Option<*mut Node> {
        if node.is_null() {
            return unsafe { Node::new(key, value) }
        }
        unsafe {
            if key < (*node).key {
                (*node).left = self.insert_node((*node).left, key, value)?;
            } else if key > (*node).key {
                (*node).right = self.insert_node((*node).right, key, value)?;
            } else {
                return None;
                // if we wanted to update the value for an existing key:
                // (*node).value.free();
                // (*node).value = value;
                // return Some(node);
            }

            (*node).update_height();
            Self::rebalance(node)
        }
    }

    pub fn insert(&mut self, key: u64, value: PoorString) -> bool {
        match unsafe { self.insert_node(self.root, key, value) } {
            Some(new_root) => {
                self.root = new_root;
                true
            }
            None => false,
        }
    }

    pub fn get(&self, key: u64) -> Option<&PoorString> {
        let mut current = self.root;
        while !current.is_null() {
            unsafe {
                if key == (*current).key {
                    return Some(&(*current).value);
                } else if key < (*current).key {
                    current = (*current).left;
                } else {
                    current = (*current).right;
                }
            }
        }
        None
    }

    pub fn contains(&self, key: u64) -> bool {
        self.get(key).is_some()
    }

    unsafe fn min_node(mut node: *mut Node) -> *mut Node {
        unsafe {
            while !(*node).left.is_null() {
                node = (*node).left;
            }
            node
        }
    }

    unsafe fn get_child_or_null(node: *mut Node) -> *mut Node {
        unsafe {
            if !(*node).left.is_null() {
                (*node).left
            } else {
                (*node).right
            }
        }
    }

    unsafe fn delete_node(&self, node: *mut Node, key: u64) -> Option<*mut Node> {
        if node.is_null() {
            return None;
        }
        unsafe {
            if key < (*node).key {
                (*node).left = self.delete_node((*node).left, key)?;
            } else if key > (*node).key {
                (*node).right = self.delete_node((*node).right, key)?;
            } else if (*node).left.is_null() || (*node).right.is_null() { // 1 or 0 children
                let result_node = Self::get_child_or_null(node);
                Node::free(node);
                return Some(result_node);
            } else { // 2 children
                let successor_node = Self::min_node((*node).right);
                Node::move_key_value_from_to(successor_node, node);
                (*node).right = self.delete_node((*node).right, (*successor_node).key)?;
            }

            (*node).update_height();
            Self::rebalance(node)
        }
    }

    pub fn delete(&mut self, key: u64) -> bool {
        match unsafe { self.delete_node(self.root, key) } {
            Some(new_root) => {
                self.root = new_root;
                true
            }
            None => false,
        }
    }
}

#[macro_export]
macro_rules! avl {
    ($( $key:expr => $val:expr ),* ) => {{
        (|| -> Option<AVLTree> {
            unsafe {
                let mut tree = AVLTree::new();
                $(
                    let ps = PoorString::new($val.as_bytes())?;
                     if !tree.insert($key, ps) {
                        // if the data provided is invalid, we do not create the tree
                        return None;
                    }
                )*
                Some(tree)
            }
        })()
    }};
}

#[unsafe(no_mangle)]
pub extern "C" fn avl_create() -> *mut AVLTree {
    let layout = Layout::new::<AVLTree>();
    let ptr = unsafe { std::alloc::alloc(layout) } as *mut AVLTree;
    if ptr.is_null() {
        return std::ptr::null_mut();
    }
    unsafe { (*ptr).root = std::ptr::null_mut(); }
    ptr
}

/// # Safety
/// `tree` must be a pointer to a properly allocated AVL tree created by `avl_create`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn avl_free(tree: *mut AVLTree) {
    if tree.is_null() {
        return;
    }
    unsafe {
        (*tree).free();
        let layout = Layout::new::<AVLTree>();
        std::alloc::dealloc(tree as *mut u8, layout);
    }
}

/// # Safety
/// `tree` must be a pointer to a properly allocated AVL tree created by `avl_create`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn avl_insert(tree: *mut AVLTree, key: u64, value: *const c_char) -> i32 {
    if tree.is_null() || value.is_null() {
        return -1;
    }
    let val = match unsafe { PoorString::new(CStr::from_ptr(value).to_bytes()) } {
        Some(ps) => ps,
        None => return -1,
    };
    match unsafe { (*tree).insert(key, val) } {
        true => 0,
        false => -1,
    }
}

/// # Safety
/// `tree` must be a pointer to a properly allocated AVL tree created by `avl_create`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn avl_contains(tree: *mut AVLTree, key: u64) -> i32 {
    if tree.is_null() {
        return 0;
    }
    if unsafe { (*tree).contains(key) } {
        1
    } else {
        0
    }
}

/// # Safety
/// `tree` must be a pointer to a properly allocated AVL tree created by `avl_create`.
/// The returned pointer to a c-string must be freed by calling `free_string`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn avl_get(tree: *mut AVLTree, key: u64) -> *const c_char {
    if tree.is_null() {
        return std::ptr::null();
    }
    match unsafe { (*tree).get(key) } {
        Some(string) => match string.as_str() {
            Some(s) => {
                let layout = match Layout::from_size_align(s.len() + 1, 1) {
                    Ok(l) => l,
                    Err(_) => return std::ptr::null(),
                };
                let ptr = unsafe { std::alloc::alloc(layout) };
                if ptr.is_null() {
                    return std::ptr::null();
                }
                unsafe {
                    std::ptr::copy_nonoverlapping(s.as_ptr(), ptr, s.len());
                    *ptr.add(s.len()) = 0; // null terminator
                }
                ptr as *const c_char
            }
            None => std::ptr::null(),
        },
        None => std::ptr::null(),
    }
}

/// # Safety
/// `s` must be a pointer to a properly allocated c-string created by `avl_get`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn free_string(s: *mut c_char) {
    if !s.is_null() {
        let len = unsafe { CStr::from_ptr(s).to_bytes().len() };
        if let Ok(layout) = Layout::from_size_align(len + 1, 1) {
            unsafe { std::alloc::dealloc(s as *mut u8, layout); }
        } // in case of Err we do nothing
    }
}

/// # Safety
/// `tree` must be a pointer to a properly allocated AVL tree created by `avl_create`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn avl_delete(tree: *mut AVLTree, key: u64) -> i32 {
    if tree.is_null() {
        return -1;
    }
    match unsafe { (*tree).delete(key) } {
        true => 0,
        false => -1,
    }
}