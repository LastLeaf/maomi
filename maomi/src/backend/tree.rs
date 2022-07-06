use std::{
    cell::{RefCell, UnsafeCell},
    fmt::Debug,
    mem::ManuallyDrop,
    ops::{Deref, DerefMut},
    pin::Pin,
    ptr,
    rc::Rc,
};

struct ForestRel<T> {
    buf_index: usize,
    parent: *const UnsafeCell<ForestRel<T>>,
    prev_sibling: *const UnsafeCell<ForestRel<T>>,
    next_sibling: *const UnsafeCell<ForestRel<T>>,
    first_child: *const UnsafeCell<ForestRel<T>>,
    last_child: *const UnsafeCell<ForestRel<T>>,
    content: T,
}

impl<T> ForestRel<T> {
    #[inline(always)]
    unsafe fn wrap(this: &*const UnsafeCell<ForestRel<T>>) -> &mut ForestRel<T> {
        &mut *(&**this).get()
    }
}

struct ForestCtx<T> {
    buf: Vec<ManuallyDrop<Pin<Box<UnsafeCell<ForestRel<T>>>>>>, // TODO change to manual alloc without vec
    freed: Vec<usize>,
}

impl<T> ForestCtx<T> {
    fn alloc(this: &Rc<RefCell<Self>>, content: T) -> ForestTree<T> {
        let ctx = this.clone();
        let mut this = this.borrow_mut();
        let mut v = UnsafeCell::new(ForestRel {
            buf_index: 0,
            parent: ptr::null_mut(),
            prev_sibling: ptr::null_mut(),
            next_sibling: ptr::null_mut(),
            first_child: ptr::null_mut(),
            last_child: ptr::null_mut(),
            content,
        });
        let index = if let Some(index) = this.freed.pop() {
            v.get_mut().buf_index = index;
            this.buf[index] = ManuallyDrop::new(Box::pin(v));
            index
        } else {
            let index = this.buf.len();
            v.get_mut().buf_index = index;
            this.buf.push(ManuallyDrop::new(Box::pin(v)));
            index
        };
        let v_ptr = &*this.buf[index].as_ref() as *const UnsafeCell<ForestRel<T>>;
        ForestTree {
            ctx,
            joint: false,
            inner: v_ptr,
        }
    }

    fn drop_subtree(&mut self, inner: *const UnsafeCell<ForestRel<T>>) {
        let inner = unsafe { ForestRel::wrap(&inner) };
        let mut cur = inner.first_child;
        while !cur.is_null() {
            let next = unsafe { ForestRel::wrap(&cur) }.next_sibling;
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

/// A tree in a forest
///
/// At any moment, only one mutable reference can be obtained in a single tree.
pub struct ForestTree<T> {
    ctx: Rc<RefCell<ForestCtx<T>>>,
    joint: bool,
    inner: *const UnsafeCell<ForestRel<T>>,
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
    /// Create a tree in a new forest
    pub fn new_forest(content: T) -> Self {
        let ctx = ForestCtx {
            buf: vec![],
            freed: vec![],
        };
        let ctx = Rc::new(RefCell::new(ctx));
        ForestCtx::alloc(&ctx, content)
    }

    /// Get an immutable reference of the tree root
    #[inline]
    pub fn as_node<'a>(&'a self) -> ForestNode<'a, T> {
        ForestNode {
            ctx: &self.ctx,
            inner: self.inner as *const _,
        }
    }

    /// Get a mutable reference of the tree root
    #[inline]
    pub fn as_node_mut<'a>(&'a mut self) -> ForestNodeMut<'a, T> {
        ForestNodeMut {
            ctx: &self.ctx,
            inner: self.inner,
        }
    }

    fn join_into<'a, 'b>(
        mut self,
        into: &'b mut ForestNodeMut<'a, T>,
    ) -> *const UnsafeCell<ForestRel<T>> {
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

pub struct ForestNode<'a, T> {
    ctx: &'a Rc<RefCell<ForestCtx<T>>>,
    inner: *const UnsafeCell<ForestRel<T>>,
}

impl<'a, T> Clone for ForestNode<'a, T> {
    fn clone(&self) -> Self {
        Self {
            ctx: self.ctx,
            inner: self.inner,
        }
    }
}

impl<'a, T> Deref for ForestNode<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &unsafe { ForestRel::wrap(&self.inner) }.content
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

impl<'a, T> PartialEq for ForestNode<'a, T> {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl<'a, T> ForestNode<'a, T> {
    /// Get the parent node
    #[inline]
    pub fn parent(&self) -> Option<ForestNode<'a, T>> {
        let this = unsafe { ForestRel::wrap(&self.inner) };
        if this.parent.is_null() {
            return None;
        }
        Some(ForestNode {
            ctx: self.ctx,
            inner: this.parent,
        })
    }

    /// Get the first child node
    #[inline]
    pub fn first_child(&self) -> Option<ForestNode<'a, T>> {
        let this = unsafe { ForestRel::wrap(&self.inner) };
        if this.first_child.is_null() {
            return None;
        }
        Some(ForestNode {
            ctx: self.ctx,
            inner: this.first_child,
        })
    }

    /// Get the last child node
    #[inline]
    pub fn last_child(&self) -> Option<ForestNode<'a, T>> {
        let this = unsafe { ForestRel::wrap(&self.inner) };
        if this.last_child.is_null() {
            return None;
        }
        Some(ForestNode {
            ctx: self.ctx,
            inner: this.last_child,
        })
    }

    /// Get the previous sibling node
    #[inline]
    pub fn prev_sibling(&self) -> Option<ForestNode<'a, T>> {
        let this = unsafe { ForestRel::wrap(&self.inner) };
        if this.prev_sibling.is_null() {
            return None;
        }
        Some(ForestNode {
            ctx: self.ctx,
            inner: this.prev_sibling,
        })
    }

    /// Get the next sibling node
    #[inline]
    pub fn next_sibling(&self) -> Option<ForestNode<'a, T>> {
        let this = unsafe { ForestRel::wrap(&self.inner) };
        if this.next_sibling.is_null() {
            return None;
        }
        Some(ForestNode {
            ctx: self.ctx,
            inner: this.next_sibling,
        })
    }
}

pub struct ForestNodeMut<'a, T> {
    ctx: &'a Rc<RefCell<ForestCtx<T>>>,
    inner: *const UnsafeCell<ForestRel<T>>,
}

impl<'a, T> Deref for ForestNodeMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &unsafe { ForestRel::wrap(&self.inner) }.content
    }
}

impl<'a, T> DerefMut for ForestNodeMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut unsafe { ForestRel::wrap(&self.inner) }.content
    }
}

impl<'a, T: Debug> Debug for ForestNodeMut<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.as_ref())
    }
}

impl<'a, T> ForestNodeMut<'a, T> {
    /// Get an immutable reference
    #[inline(always)]
    pub fn as_ref<'b>(&'b self) -> ForestNode<'b, T> {
        ForestNode {
            ctx: self.ctx,
            inner: self.inner as *const _,
        }
    }

    /// Make a wrapped component the contained value, keeping the borrowing status
    #[inline]
    pub fn map<'b, U>(
        &'b mut self,
        f: impl FnOnce(&'b mut T) -> &'b mut U,
    ) -> ForestValueMut<'b, U> {
        ForestValueMut {
            v: f(&mut unsafe { ForestRel::wrap(&self.inner) }.content),
        }
    }

    /// Create a new tree in the same forest
    #[inline]
    pub fn new_tree(&self, content: T) -> ForestTree<T> {
        ForestCtx::alloc(self.ctx, content)
    }

    /// Append a tree as the last child node
    pub fn append(&mut self, tree: ForestTree<T>) {
        let child_ptr = tree.join_into(self);
        let parent_ptr = self.inner;
        let parent = unsafe { ForestRel::wrap(&parent_ptr) };
        let child = unsafe { ForestRel::wrap(&child_ptr) };
        child.parent = parent_ptr;
        let last_child_ptr = parent.last_child;
        if last_child_ptr.is_null() {
            parent.first_child = child_ptr;
        } else {
            let last_child = unsafe { ForestRel::wrap(&last_child_ptr) };
            child.prev_sibling = last_child_ptr;
            last_child.next_sibling = child_ptr;
        }
        parent.last_child = child_ptr;
    }

    /// Insert a tree as the previous sibling node of the current node
    ///
    /// Panics if called on tree root node.
    pub fn insert(&mut self, tree: ForestTree<T>) {
        let child_ptr = tree.join_into(self);
        let before_ptr = self.inner;
        let before = unsafe { ForestRel::wrap(&before_ptr) };
        let parent_ptr = before.parent;
        if parent_ptr.is_null() {
            panic!("Cannot insert at a tree root node");
        }
        let parent = unsafe { ForestRel::wrap(&parent_ptr) };
        let child = unsafe { ForestRel::wrap(&child_ptr) };
        child.parent = parent_ptr;
        if before.prev_sibling.is_null() {
            parent.first_child = child_ptr;
        } else {
            let prev = unsafe { ForestRel::wrap(&before.prev_sibling) };
            prev.next_sibling = child_ptr;
        }
        child.prev_sibling = before.prev_sibling;
        child.next_sibling = before_ptr;
        before.prev_sibling = child_ptr;
    }

    /// Remove the node from its parent node
    ///
    /// Panics if called on tree root node.
    pub fn detach(&mut self) -> ForestTree<T> {
        let child_ptr = self.inner;
        let child = unsafe { ForestRel::wrap(&child_ptr) };
        let parent_ptr = child.parent;
        if parent_ptr.is_null() {
            panic!("Cannot detach a tree root node");
        }
        let parent = unsafe { ForestRel::wrap(&parent_ptr) };
        let prev_ptr = child.prev_sibling;
        let next_ptr = child.next_sibling;
        if prev_ptr.is_null() {
            parent.first_child = next_ptr;
        } else {
            let prev = unsafe { ForestRel::wrap(&prev_ptr) };
            prev.next_sibling = next_ptr;
        }
        if next_ptr.is_null() {
            parent.last_child = prev_ptr;
        } else {
            let next = unsafe { ForestRel::wrap(&next_ptr) };
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

    /// Get the parent node
    #[inline]
    pub fn parent(self) -> Option<ForestNodeMut<'a, T>> {
        let this = unsafe { ForestRel::wrap(&self.inner) };
        if this.parent.is_null() {
            return None;
        }
        Some(ForestNodeMut {
            ctx: self.ctx,
            inner: this.parent,
        })
    }

    /// Get the parent node
    #[inline]
    pub fn parent_mut<'c>(&'c mut self) -> Option<ForestNodeMut<'c, T>> {
        let this = unsafe { ForestRel::wrap(&self.inner) };
        if this.parent.is_null() {
            return None;
        }
        Some(ForestNodeMut {
            ctx: self.ctx,
            inner: this.parent,
        })
    }

    /// Get the first child node
    #[inline]
    pub fn first_child(self) -> Option<ForestNodeMut<'a, T>> {
        let this = unsafe { ForestRel::wrap(&self.inner) };
        if this.first_child.is_null() {
            return None;
        }
        Some(ForestNodeMut {
            ctx: self.ctx,
            inner: this.first_child,
        })
    }

    /// Get the first child node
    #[inline]
    pub fn first_child_mut<'c>(&'c mut self) -> Option<ForestNodeMut<'c, T>> {
        let this = unsafe { ForestRel::wrap(&self.inner) };
        if this.first_child.is_null() {
            return None;
        }
        Some(ForestNodeMut {
            ctx: self.ctx,
            inner: this.first_child,
        })
    }

    /// Get the last child node
    #[inline]
    pub fn last_child(self) -> Option<ForestNodeMut<'a, T>> {
        let this = unsafe { ForestRel::wrap(&self.inner) };
        if this.last_child.is_null() {
            return None;
        }
        Some(ForestNodeMut {
            ctx: self.ctx,
            inner: this.last_child,
        })
    }

    /// Get the last child node
    #[inline]
    pub fn last_child_mut<'c>(&'c mut self) -> Option<ForestNodeMut<'c, T>> {
        let this = unsafe { ForestRel::wrap(&self.inner) };
        if this.last_child.is_null() {
            return None;
        }
        Some(ForestNodeMut {
            ctx: self.ctx,
            inner: this.last_child,
        })
    }

    /// Get the previous sibling node
    #[inline]
    pub fn prev_sibling(self) -> Option<ForestNodeMut<'a, T>> {
        let this = unsafe { ForestRel::wrap(&self.inner) };
        if this.prev_sibling.is_null() {
            return None;
        }
        Some(ForestNodeMut {
            ctx: self.ctx,
            inner: this.prev_sibling,
        })
    }

    /// Get the previous sibling node
    #[inline]
    pub fn prev_sibling_mut<'c>(&'c mut self) -> Option<ForestNodeMut<'c, T>> {
        let this = unsafe { ForestRel::wrap(&self.inner) };
        if this.prev_sibling.is_null() {
            return None;
        }
        Some(ForestNodeMut {
            ctx: self.ctx,
            inner: this.prev_sibling,
        })
    }

    /// Get the next sibling node
    #[inline]
    pub fn next_sibling(self) -> Option<ForestNodeMut<'a, T>> {
        let this = unsafe { ForestRel::wrap(&self.inner) };
        if this.next_sibling.is_null() {
            return None;
        }
        Some(ForestNodeMut {
            ctx: self.ctx,
            inner: this.next_sibling,
        })
    }

    /// Get the next sibling node
    #[inline]
    pub fn next_sibling_mut<'c>(&'c mut self) -> Option<ForestNodeMut<'c, T>> {
        let this = unsafe { ForestRel::wrap(&self.inner) };
        if this.next_sibling.is_null() {
            return None;
        }
        Some(ForestNodeMut {
            ctx: self.ctx,
            inner: this.next_sibling,
        })
    }
}

pub struct ForestValue<'a, T> {
    v: &'a T,
}

impl<'a, T> Deref for ForestValue<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.v
    }
}

impl<'a, T> ForestValue<'a, T> {
    pub fn map<U>(&'a self, f: impl FnOnce(&'a T) -> &'a U) -> ForestValue<'a, U> {
        ForestValue { v: f(self.v) }
    }
}

pub struct ForestValueMut<'a, T> {
    v: &'a mut T,
}

impl<'a, T> Deref for ForestValueMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.v
    }
}

impl<'a, T> DerefMut for ForestValueMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.v
    }
}

impl<'a, T> ForestValueMut<'a, T> {
    pub fn as_ref<'b>(&'b self) -> ForestValue<'b, T> {
        ForestValue { v: self.v }
    }

    pub fn map<'b, U>(
        &'b mut self,
        f: impl FnOnce(&'b mut T) -> &'b mut U,
    ) -> ForestValueMut<'b, U> {
        ForestValueMut { v: f(self.v) }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn check_pointers<'a, T: PartialEq>(tree: &mut ForestTree<T>) {
        fn rec<'a, T: PartialEq>(node: &ForestNode<'a, T>) {
            let mut prev = None;
            let mut cur_option = node.first_child();
            while let Some(cur) = cur_option {
                assert!(cur.parent().as_ref() == Some(node));
                assert!(cur.prev_sibling() == prev);
                rec(&cur);
                cur_option = cur.next_sibling();
                prev = Some(cur);
            }
            assert!(node.last_child() == prev);
        }
        let node = tree.as_node();
        assert!(node.parent() == None);
        assert!(node.next_sibling() == None);
        assert!(node.prev_sibling() == None);
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
            assert_eq!(
                format!("{:?}", n1),
                r#"ForestNode(1) [ForestNode(2) [ForestNode(3) [], ForestNode(4) []]]"#
            );
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
                n2.first_child_mut()
                    .unwrap()
                    .next_sibling_mut()
                    .unwrap()
                    .insert(n5);
            }
            n1.append(n2);
            assert_eq!(
                format!("{:?}", n1),
                r#"ForestNode(1) [ForestNode(2) [ForestNode(4) [], ForestNode(5) [], ForestNode(3) []]]"#
            );
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
            assert_eq!(
                format!("{:?}", n1),
                r#"ForestNode(1) [ForestNode(2) [ForestNode(3) [], ForestNode(4) [], ForestNode(5) []]]"#
            );
            let n4 = n1
                .first_child_mut()
                .unwrap()
                .first_child_mut()
                .unwrap()
                .next_sibling_mut()
                .unwrap()
                .detach();
            assert_eq!(
                format!("{:?}", n1),
                r#"ForestNode(1) [ForestNode(2) [ForestNode(3) [], ForestNode(5) []]]"#
            );
            assert_eq!(format!("{:?}", n4), r#"ForestNode(4) []"#);
            let n2 = n1.first_child_mut().unwrap().detach();
            assert_eq!(format!("{:?}", n1), r#"ForestNode(1) []"#);
            assert_eq!(
                format!("{:?}", n2),
                r#"ForestNode(2) [ForestNode(3) [], ForestNode(5) []]"#
            );
        }
        check_pointers(&mut tree);
    }
}
