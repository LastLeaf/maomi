use std::{
    cell::RefCell,
    fmt::Debug,
    mem::ManuallyDrop,
    ops::{Deref, DerefMut},
    pin::Pin,
    ptr,
    rc::Rc,
};

struct ForestCtx<T> {
    buf: Vec<ManuallyDrop<Pin<Box<ForestNodeInner<T>>>>>,
    freed: Vec<usize>,
}

struct ForestNodeInner<T> {
    buf_index: usize,
    parent: *mut ForestNodeInner<T>,
    prev_sibling: *mut ForestNodeInner<T>,
    next_sibling: *mut ForestNodeInner<T>,
    first_child: *mut ForestNodeInner<T>,
    last_child: *mut ForestNodeInner<T>,
    content: T,
}

impl<T> ForestCtx<T> {
    fn alloc(this: &Rc<RefCell<Self>>, content: T) -> ForestTree<T> {
        let ctx = this.clone();
        let mut this = this.borrow_mut();
        let mut v = ForestNodeInner {
            buf_index: 0,
            parent: ptr::null_mut(),
            prev_sibling: ptr::null_mut(),
            next_sibling: ptr::null_mut(),
            first_child: ptr::null_mut(),
            last_child: ptr::null_mut(),
            content,
        };
        let index = if let Some(index) = this.freed.pop() {
            v.buf_index = index;
            this.buf[index] = ManuallyDrop::new(Box::pin(v));
            index
        } else {
            let index = this.buf.len();
            v.buf_index = index;
            this.buf.push(ManuallyDrop::new(Box::pin(v)));
            index
        };
        let v_ptr = unsafe { this.buf[index].as_mut().get_unchecked_mut() } as *mut _;
        ForestTree {
            ctx,
            joint: false,
            inner: v_ptr,
        }
    }

    fn drop_subtree(&mut self, inner: *mut ForestNodeInner<T>) {
        let inner = unsafe { Pin::new_unchecked(&mut *inner) };
        let mut cur = inner.first_child;
        while !cur.is_null() {
            let next = unsafe { Pin::new_unchecked(&mut *cur) }.next_sibling;
            self.drop_subtree(cur);
            cur = next;
        }
        let index = inner.buf_index;
        unsafe {
            ManuallyDrop::drop(&mut self.buf[index]);
        }
        self.freed.push(index);
    }
}

// We should guarantee that -
// at any moment, only one mut ref can be obtained in a single tree
pub(crate) struct ForestTree<T> {
    ctx: Rc<RefCell<ForestCtx<T>>>,
    joint: bool,
    inner: *mut ForestNodeInner<T>,
}

impl<T> Drop for ForestTree<T> {
    fn drop(&mut self) {
        if self.joint {
            return;
        }
        self.ctx.borrow_mut().drop_subtree(self.inner);
    }
}

impl<T> ForestTree<T> {
    pub(crate) fn new_forest(content: T) -> Self {
        let ctx = ForestCtx {
            buf: vec![],
            freed: vec![],
        };
        let ctx = Rc::new(RefCell::new(ctx));
        ForestCtx::alloc(&ctx, content)
    }

    pub(crate) fn as_node<'a>(&'a self) -> ForestNode<'a, T> {
        ForestNode {
            ctx: &self.ctx,
            inner: self.inner as *const _,
        }
    }

    pub(crate) fn as_node_mut<'a>(&'a mut self) -> ForestNodeMut<'a, T> {
        ForestNodeMut {
            ctx: &self.ctx,
            inner: self.inner,
        }
    }

    fn join_into<'a, 'b>(mut self, into: &'b mut ForestNodeMut<'a, T>) -> *mut ForestNodeInner<T> {
        if !Rc::ptr_eq(&self.ctx, into.ctx) {
            panic!("Cannot join two trees in different forest");
        }
        self.joint = true;
        self.inner
    }
}

impl<T: Debug> Debug for ForestTree<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.as_node())
    }
}

#[derive(Clone, Copy)]
pub(crate) struct ForestNode<'a, T> {
    ctx: &'a Rc<RefCell<ForestCtx<T>>>,
    inner: *const ForestNodeInner<T>,
}

impl<'a, T> Deref for ForestNode<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &unsafe { &*self.inner }.content
    }
}

impl<'a, T: Debug> Debug for ForestNode<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let content: &T = &**self;
        write!(f, "ForestNode({:?}) [", content)?;
        let mut cur = self.first_child();
        if let Some(c) = cur {
            write!(f, "{:?}", c)?;
            cur = c.next_sibling();
            while let Some(c) = cur {
                write!(f, ", {:?}", c)?;
                cur = c.next_sibling();
            }
        }
        write!(f, "]")?;
        Ok(())
    }
}

impl<'a, T> ForestNode<'a, T> {
    pub(crate) fn first_child(&self) -> Option<ForestNode<'a, T>> {
        let this = unsafe { &*self.inner };
        if this.first_child.is_null() {
            return None;
        }
        Some(ForestNode {
            ctx: self.ctx,
            inner: this.first_child,
        })
    }

    pub(crate) fn next_sibling(&self) -> Option<ForestNode<'a, T>> {
        let this = unsafe { &*self.inner };
        if this.next_sibling.is_null() {
            return None;
        }
        Some(ForestNode {
            ctx: self.ctx,
            inner: this.next_sibling,
        })
    }
}

pub(crate) struct ForestNodeMut<'a, T> {
    ctx: &'a Rc<RefCell<ForestCtx<T>>>,
    inner: *mut ForestNodeInner<T>,
}

impl<'a, T> Deref for ForestNodeMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &unsafe { &*self.inner }.content
    }
}

impl<'a, T> DerefMut for ForestNodeMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut unsafe { &mut *self.inner }.content
    }
}

impl<'a, T: Debug> Debug for ForestNodeMut<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.as_ref())
    }
}

impl<'a, T> ForestNodeMut<'a, T> {
    pub(crate) fn as_ref<'b>(&'b self) -> ForestNode<'b, T> {
        ForestNode {
            ctx: self.ctx,
            inner: self.inner as *const _,
        }
    }

    pub(crate) fn new_tree(&self, content: T) -> ForestTree<T> {
        ForestCtx::alloc(self.ctx, content)
    }

    pub(crate) fn append(&mut self, tree: ForestTree<T>) {
        let child_ptr = tree.join_into(self);
        let parent_ptr = self.inner;
        let parent = unsafe { &mut *parent_ptr };
        let child = unsafe { &mut *child_ptr };
        child.parent = parent_ptr;
        let last_child_ptr = parent.last_child;
        if last_child_ptr.is_null() {
            parent.first_child = child_ptr;
        } else {
            let last_child = unsafe { &mut *last_child_ptr };
            child.prev_sibling = last_child_ptr;
            last_child.next_sibling = child_ptr;
        }
        parent.last_child = child_ptr;
    }

    pub(crate) fn insert(&mut self, tree: ForestTree<T>) {
        let child_ptr = tree.join_into(self);
        let before_ptr = self.inner;
        let before = unsafe { &mut *before_ptr };
        let parent_ptr = before.parent;
        if parent_ptr.is_null() {
            panic!("Cannot insert at a tree root node");
        }
        let parent = unsafe { &mut *parent_ptr };
        let child = unsafe { &mut *child_ptr };
        child.parent = parent_ptr;
        if before.prev_sibling.is_null() {
            parent.first_child = child_ptr;
        } else {
            let prev = unsafe { &mut *before.prev_sibling };
            prev.next_sibling = child_ptr;
        }
        child.prev_sibling = before.prev_sibling;
        child.next_sibling = before_ptr;
        before.prev_sibling = child_ptr;
    }

    pub(crate) fn detach(&mut self) -> ForestTree<T> {
        let child_ptr = self.inner;
        let child = unsafe { &mut *child_ptr };
        let parent_ptr = child.parent;
        if parent_ptr.is_null() {
            panic!("Cannot detach a tree root node");
        }
        let parent = unsafe { &mut *parent_ptr };
        let prev_ptr = child.prev_sibling;
        let next_ptr = child.next_sibling;
        if prev_ptr.is_null() {
            parent.first_child = next_ptr;
        } else {
            let prev = unsafe { &mut *prev_ptr };
            prev.next_sibling = next_ptr;
        }
        if next_ptr.is_null() {
            parent.last_child = prev_ptr;
        } else {
            let next = unsafe { &mut *next_ptr };
            next.prev_sibling = prev_ptr;
        }
        child.parent = ptr::null_mut();
        child.next_sibling = ptr::null_mut();
        child.prev_sibling = ptr::null_mut();
        ForestTree {
            ctx: self.ctx.clone(),
            joint: false,
            inner: child_ptr,
        }
    }

    pub(crate) fn first_child<'c>(&'c self) -> Option<ForestNode<'c, T>> {
        let this = unsafe { &mut *self.inner };
        if this.first_child.is_null() {
            return None;
        }
        Some(ForestNode {
            ctx: self.ctx,
            inner: this.first_child,
        })
    }

    pub(crate) fn first_child_mut<'c>(&'c mut self) -> Option<ForestNodeMut<'c, T>> {
        let this = unsafe { &mut *self.inner };
        if this.first_child.is_null() {
            return None;
        }
        Some(ForestNodeMut {
            ctx: self.ctx,
            inner: this.first_child,
        })
    }

    pub(crate) fn next_sibling<'c>(&'c self) -> Option<ForestNode<'c, T>> {
        let this = unsafe { &mut *self.inner };
        if this.next_sibling.is_null() {
            return None;
        }
        Some(ForestNode {
            ctx: self.ctx,
            inner: this.next_sibling,
        })
    }

    pub(crate) fn next_sibling_mut<'c>(&'c mut self) -> Option<ForestNodeMut<'c, T>> {
        let this = unsafe { &mut *self.inner };
        if this.next_sibling.is_null() {
            return None;
        }
        Some(ForestNodeMut {
            ctx: self.ctx,
            inner: this.next_sibling,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn check_pointers<'a, T: PartialEq>(tree: &mut ForestTree<T>) {
        fn rec<'a, T: PartialEq>(node: &ForestNode<'a, T>) {
            let mut prev = ptr::null();
            let mut cur_option = node.first_child();
            while let Some(cur) = cur_option {
                assert_eq!(unsafe { &*cur.inner }.parent as *const _, node.inner);
                assert_eq!(unsafe { &*cur.inner }.prev_sibling as *const _, prev);
                rec(&cur);
                prev = cur.inner;
                cur_option = cur.next_sibling();
            }
            assert_eq!(unsafe { &*node.inner }.last_child as *const _, prev);
        }
        let node = tree.as_node();
        assert!(unsafe { &*node.inner }.parent.is_null());
        assert!(unsafe { &*node.inner }.next_sibling.is_null());
        assert!(unsafe { &*node.inner }.prev_sibling.is_null());
        rec(&node);
    }

    #[test]
    fn append() {
        let mut tree = ForestTree::new_forest(1);
        {
            let mut n1 = tree.as_node_mut();
            let mut n2 = n1.new_tree(2);
            {
                let mut n2 = n2.as_node_mut();
                let n3 = n1.new_tree(3);
                n2.append(n3);
                let n4 = n1.new_tree(4);
                n2.append(n4);
            }
            n1.append(n2);
            assert_eq!(format!("{:?}", n1), r#"ForestNode(1) [ForestNode(2) [ForestNode(3) [], ForestNode(4) []]]"#);
        }
        check_pointers(&mut tree);
    }

    #[test]
    fn insert() {
        let mut tree = ForestTree::new_forest(1);
        {
            let mut n1 = tree.as_node_mut();
            let mut n2 = n1.new_tree(2);
            {
                let mut n2 = n2.as_node_mut();
                let n3 = n1.new_tree(3);
                n2.append(n3);
                let n4 = n1.new_tree(4);
                n2.first_child_mut().unwrap().insert(n4);
                let n5 = n1.new_tree(5);
                n2.first_child_mut().unwrap().next_sibling_mut().unwrap().insert(n5);
            }
            n1.append(n2);
            assert_eq!(format!("{:?}", n1), r#"ForestNode(1) [ForestNode(2) [ForestNode(4) [], ForestNode(5) [], ForestNode(3) []]]"#);
        }
        check_pointers(&mut tree);
    }

    #[test]
    fn detach() {
        let mut tree = ForestTree::new_forest(1);
        {
            let mut n1 = tree.as_node_mut();
            let mut n2 = n1.new_tree(2);
            {
                let mut n2 = n2.as_node_mut();
                let n3 = n1.new_tree(3);
                n2.append(n3);
                let n4 = n1.new_tree(4);
                n2.append(n4);
                let n5 = n1.new_tree(5);
                n2.append(n5);
            }
            n1.append(n2);
            assert_eq!(format!("{:?}", n1), r#"ForestNode(1) [ForestNode(2) [ForestNode(3) [], ForestNode(4) [], ForestNode(5) []]]"#);
            let n4 = n1.first_child_mut().unwrap().first_child_mut().unwrap().next_sibling_mut().unwrap().detach();
            assert_eq!(format!("{:?}", n1), r#"ForestNode(1) [ForestNode(2) [ForestNode(3) [], ForestNode(5) []]]"#);
            assert_eq!(format!("{:?}", n4), r#"ForestNode(4) []"#);
            let n2 = n1.first_child_mut().unwrap().detach();
            assert_eq!(format!("{:?}", n1), r#"ForestNode(1) []"#);
            assert_eq!(format!("{:?}", n2), r#"ForestNode(2) [ForestNode(3) [], ForestNode(5) []]"#);
        }
        check_pointers(&mut tree);
    }
}
