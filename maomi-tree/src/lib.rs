use std::{
    cell::{Cell, RefCell},
    fmt::Debug,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    rc::Rc,
};

const SLICE_ITEMS: usize = 256;

mod mem;
use mem::{SliceAlloc, SliceRc, SliceWeak};

struct ForestCtx<T> {
    slice_alloc: RefCell<SliceAlloc<ForestRel<T>, SLICE_ITEMS>>,
    ref_count: Cell<usize>,
    mut_count: Cell<usize>,
}

impl<T> ForestCtx<T> {
    pub fn new_node(self: &mut Rc<Self>, content: T) -> ForestNodeRc<T> {
        let ctx = self.clone();
        let slice = self.slice_alloc.borrow_mut().alloc(ForestRel {
            ctx,
            parent: None,
            prev_sibling: None,
            next_sibling: None,
            first_child: None,
            last_child: None,
            content,
        });
        ForestNodeRc { inner: slice }
    }
}

struct ForestRel<T> {
    ctx: Rc<ForestCtx<T>>,
    parent: Option<SliceWeak<ForestRel<T>, SLICE_ITEMS>>,
    prev_sibling: Option<SliceWeak<ForestRel<T>, SLICE_ITEMS>>,
    next_sibling: Option<SliceRc<ForestRel<T>, SLICE_ITEMS>>,
    first_child: Option<SliceRc<ForestRel<T>, SLICE_ITEMS>>,
    last_child: Option<SliceWeak<ForestRel<T>, SLICE_ITEMS>>,
    content: T,
}

/// A node in a forest
pub struct ForestNodeRc<T> {
    inner: SliceRc<ForestRel<T>, SLICE_ITEMS>,
}

impl<T> Clone for ForestNodeRc<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T> ForestNodeRc<T> {
    /// Create a node in a new forest
    #[inline]
    pub fn new_forest(content: T) -> Self {
        let mut ctx = Rc::new(ForestCtx {
            slice_alloc: RefCell::new(SliceAlloc::new()),
            ref_count: Cell::new(0),
            mut_count: Cell::new(0),
        });
        ForestCtx::new_node(&mut ctx, content)
    }

    /// Get an immutable reference of the node
    #[inline]
    pub fn borrow<'a>(&'a self) -> ForestNode<'a, T> {
        let inner = &self.inner;
        let ctx = &unsafe { inner.data_ref() }.ctx;
        if ctx.mut_count.get() > 0 {
            panic!(
                "Cannot borrow the forest node when a node is mutably borrowed in the same forest"
            )
        }
        ctx.ref_count.set(ctx.ref_count.get() + 1);
        ForestNode { inner }
    }

    /// Get a mutable reference of the tree root
    #[inline]
    pub fn borrow_mut<'a>(&'a self) -> ForestNodeMut<'a, T> {
        let inner = self.inner.clone();
        let ctx = &unsafe { inner.data_ref() }.ctx;
        if ctx.ref_count.get() > 0 || ctx.mut_count.get() > 0 {
            panic!(
                "Cannot mutably borrow the forest node when a node is borrowed in the same forest"
            )
        }
        ctx.mut_count.set(1);
        ForestNodeMut {
            inner,
            _phantom: PhantomData,
        }
    }

    /// Get a token
    pub fn token(&self) -> ForestToken {
        ForestToken {
            inner: self.inner.weak().leak(),
        }
    }

    /// Check if two nodes are the same
    #[inline]
    pub fn ptr_eq(&self, rhs: &Self) -> bool {
        self.inner.mem() == rhs.inner.mem()
    }
}

/// A static ref to a `ForestNodeRc<T>`
pub struct ForestToken {
    inner: *const (),
}

impl Drop for ForestToken {
    fn drop(&mut self) {
        unsafe { SliceWeak::revoke_leaked(self.inner) };
    }
}

impl Debug for ForestToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ForestToken").finish()
    }
}

pub struct ForestNode<'a, T> {
    inner: &'a SliceRc<ForestRel<T>, SLICE_ITEMS>,
}

impl<'a, T> Drop for ForestNode<'a, T> {
    fn drop(&mut self) {
        let ctx = &unsafe { self.inner.data_ref() }.ctx;
        ctx.ref_count.set(ctx.ref_count.get() - 1);
    }
}

impl<'a, T> Clone for ForestNode<'a, T> {
    fn clone(&self) -> Self {
        let ctx = &unsafe { self.inner.data_ref() }.ctx;
        ctx.ref_count.set(ctx.ref_count.get() + 1);
        Self { inner: self.inner }
    }
}

impl<'a, T> Deref for ForestNode<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &unsafe { self.inner.data_ref() }.content
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
    unsafe fn borrow_unchecked<'b>(
        &self,
        another: &'b SliceRc<ForestRel<T>, SLICE_ITEMS>,
    ) -> ForestNode<'b, T> {
        let ctx = &{ another.data_ref() }.ctx;
        ctx.ref_count.set(ctx.ref_count.get() + 1);
        ForestNode { inner: another }
    }

    /// Borrow another node in the same forest
    #[inline]
    pub fn borrow<'b>(&self, target: &'b ForestNodeRc<T>) -> ForestNode<'b, T> {
        unsafe {
            if !Rc::ptr_eq(
                &{ self.inner.data_ref() }.ctx,
                &{ &*target.inner.data_ref() }.ctx,
            ) {
                panic!("The target node is not in the same forest")
            }
            self.borrow_unchecked(&target.inner)
        }
    }

    /// Get the `ForestNodeRc` of current node
    #[inline]
    pub fn rc(&self) -> ForestNodeRc<T> {
        ForestNodeRc {
            inner: self.inner.clone(),
        }
    }

    /// Check if two nodes are the same
    #[inline]
    pub fn ptr_eq(&self, rhs: &Self) -> bool {
        self.inner.mem() == rhs.inner.mem()
    }

    /// Get the parent node
    #[inline]
    pub fn parent_rc(&self) -> Option<ForestNodeRc<T>> {
        let this = unsafe { self.inner.data_ref() };
        this.parent
            .as_ref()
            .and_then(|x| x.rc())
            .map(|x| ForestNodeRc { inner: x })
    }

    /// Get the next sibling node
    #[inline]
    pub fn first_child_rc(&self) -> Option<ForestNodeRc<T>> {
        let this = unsafe { self.inner.data_ref() };
        this.first_child
            .as_ref()
            .map(|x| ForestNodeRc { inner: x.clone() })
    }

    /// Get the first child node
    #[inline]
    pub fn first_child(&self) -> Option<ForestNode<'a, T>> {
        let this = unsafe { self.inner.data_ref() };
        this.first_child
            .as_ref()
            .map(|x| unsafe { self.borrow_unchecked(x) })
    }

    /// Get the last child node
    #[inline]
    pub fn last_child_rc(&self) -> Option<ForestNodeRc<T>> {
        let this = unsafe { self.inner.data_ref() };
        this.last_child
            .as_ref()
            .and_then(|x| x.rc())
            .map(|x| ForestNodeRc { inner: x })
    }

    /// Get the previous sibling node
    #[inline]
    pub fn prev_sibling_rc(&self) -> Option<ForestNodeRc<T>> {
        let this = unsafe { self.inner.data_ref() };
        this.prev_sibling
            .as_ref()
            .and_then(|x| x.rc())
            .map(|x| ForestNodeRc { inner: x })
    }

    /// Get the next sibling node
    #[inline]
    pub fn next_sibling_rc(&self) -> Option<ForestNodeRc<T>> {
        let this = unsafe { self.inner.data_ref() };
        this.next_sibling
            .as_ref()
            .map(|x| ForestNodeRc { inner: x.clone() })
    }

    /// Get the next sibling node
    #[inline]
    pub fn next_sibling(&self) -> Option<ForestNode<'a, T>> {
        let this = unsafe { self.inner.data_ref() };
        this.next_sibling
            .as_ref()
            .map(|x| unsafe { self.borrow_unchecked(x) })
    }
}

pub struct ForestNodeMut<'a, T> {
    inner: SliceRc<ForestRel<T>, SLICE_ITEMS>,
    _phantom: PhantomData<&'a ()>,
}

impl<'a, T> Drop for ForestNodeMut<'a, T> {
    fn drop(&mut self) {
        let ctx = &unsafe { self.inner.data_ref() }.ctx;
        ctx.mut_count.set(ctx.mut_count.get() - 1);
    }
}

impl<'a, T> Deref for ForestNodeMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &unsafe { self.inner.data_ref() }.content
    }
}

impl<'a, T> DerefMut for ForestNodeMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut unsafe { self.inner.data_mut() }.content
    }
}

impl<'a, T: Debug> Debug for ForestNodeMut<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.as_ref())
    }
}

impl<'a, T> ForestNodeMut<'a, T> {
    unsafe fn borrow_mut_unchecked<'b>(
        &'b self,
        inner: &SliceRc<ForestRel<T>, SLICE_ITEMS>,
    ) -> ForestNodeMut<'b, T> {
        let ctx = &{ &*inner.data_ref() }.ctx;
        ctx.mut_count.set(ctx.mut_count.get() + 1);
        ForestNodeMut {
            inner: inner.clone(),
            _phantom: PhantomData,
        }
    }

    /// Borrow another node in the same forest
    #[inline]
    pub fn borrow_mut<'b>(&'b mut self, target: &'b ForestNodeRc<T>) -> ForestNodeMut<'b, T> {
        unsafe {
            if !Rc::ptr_eq(
                &{ self.inner.data_ref() }.ctx,
                &{ &*target.inner.data_ref() }.ctx,
            ) {
                panic!("The target node is not in the same forest")
            }
            self.borrow_mut_unchecked(&target.inner)
        }
    }

    /// Borrow another node with a token
    ///
    /// The node which the token pointed to must be in the same forest and still has a valid `ForestNodeRc` .
    #[inline]
    pub fn resolve_token<'b>(&'b mut self, target: &ForestToken) -> ForestNodeRc<T> {
        let weak = unsafe { SliceWeak::<ForestRel<T>, SLICE_ITEMS>::from_leaked(target.inner) };
        weak.clone().leak();
        let rc = weak.rc().expect("The target node has been released");
        ForestNodeRc { inner: rc }
    }

    /// Borrow another node with a token
    ///
    /// The node which the token pointed to must be in the same forest and still has a valid `ForestNodeRc` .
    #[inline]
    pub fn borrow_mut_token<'b>(&'b mut self, target: &ForestToken) -> ForestNodeMut<'b, T> {
        let weak = unsafe { SliceWeak::<ForestRel<T>, SLICE_ITEMS>::from_leaked(target.inner) };
        weak.clone().leak();
        let rc = weak.rc().expect("The target node has been released");
        unsafe { self.borrow_mut_unchecked(&rc) }
    }

    /// Get an immutable reference
    #[inline(always)]
    pub fn as_ref<'b>(&'b self) -> ForestNode<'b, T> {
        let ctx = &unsafe { self.inner.data_ref() }.ctx;
        ctx.ref_count.set(ctx.ref_count.get() + 1);
        ForestNode { inner: &self.inner }
    }

    /// Make a wrapped component the contained value, keeping the borrowing status
    #[inline]
    pub fn map<'b, U>(
        &'b mut self,
        f: impl FnOnce(&'b mut T) -> &'b mut U,
    ) -> ForestValueMut<'b, U> {
        ForestValueMut { v: f(&mut **self) }
    }

    /// Get the parent node
    #[inline]
    pub fn parent_rc(&self) -> Option<ForestNodeRc<T>> {
        let this = unsafe { self.inner.data_ref() };
        this.parent
            .as_ref()
            .and_then(|x| x.rc())
            .map(|x| ForestNodeRc { inner: x })
    }

    /// Get the first child node
    #[inline]
    pub fn first_child_rc(&mut self) -> Option<ForestNodeRc<T>> {
        let this = unsafe { self.inner.data_ref() };
        this.first_child
            .as_ref()
            .map(|x| ForestNodeRc { inner: x.clone() })
    }

    /// Get the first child node
    #[inline]
    pub fn first_child_mut<'b>(&'b mut self) -> Option<ForestNodeMut<'b, T>> {
        let this = unsafe { self.inner.data_ref() };
        this.first_child
            .as_ref()
            .map(|x| unsafe { self.borrow_mut_unchecked(x) })
    }

    /// Get the last child node
    #[inline]
    pub fn last_child_rc(&self) -> Option<ForestNodeRc<T>> {
        let this = unsafe { self.inner.data_ref() };
        this.last_child
            .as_ref()
            .and_then(|x| x.rc())
            .map(|x| ForestNodeRc { inner: x })
    }

    /// Get the previous sibling node
    #[inline]
    pub fn prev_sibling_rc(&self) -> Option<ForestNodeRc<T>> {
        let this = unsafe { self.inner.data_ref() };
        this.prev_sibling
            .as_ref()
            .and_then(|x| x.rc())
            .map(|x| ForestNodeRc { inner: x })
    }

    /// Get the next sibling node
    #[inline]
    pub fn next_sibling_rc(&mut self) -> Option<ForestNodeRc<T>> {
        let this = unsafe { self.inner.data_ref() };
        this.next_sibling
            .as_ref()
            .map(|x| ForestNodeRc { inner: x.clone() })
    }

    /// Get the next sibling node
    #[inline]
    pub fn next_sibling_mut<'b>(&'b mut self) -> Option<ForestNodeMut<'b, T>> {
        let this = unsafe { self.inner.data_ref() };
        this.next_sibling
            .as_ref()
            .map(|x| unsafe { self.borrow_mut_unchecked(x) })
    }

    /// Create a new tree in the same forest
    #[inline]
    pub fn new_tree(&mut self, content: T) -> ForestNodeRc<T> {
        let ctx = &mut unsafe { self.inner.data_mut() }.ctx;
        ctx.new_node(content)
    }

    fn check_insertion(
        &self,
        parent: &SliceWeak<ForestRel<T>, SLICE_ITEMS>,
        target: &ForestNodeRc<T>,
    ) {
        let self_data = unsafe { self.inner.data_ref() };
        let data = unsafe { &*target.inner.data_ref() };
        if !Rc::ptr_eq(&self_data.ctx, &data.ctx) {
            panic!("The child node is not in the same forest")
        }
        if data.parent.is_some() {
            panic!("The child node already has a parent")
        }
        if target.inner.mem() == parent.mem() {
            panic!("The child node is the same as the parent")
        }
    }

    /// Append a tree as the last child node
    #[inline]
    pub fn append(&mut self, target: &ForestNodeRc<T>) {
        let parent_ptr = &self.inner;
        self.check_insertion(&parent_ptr.weak(), target);
        let parent = unsafe { &mut *parent_ptr.data_mut() };
        let child_ptr = &target.inner;
        let child = unsafe { &mut *child_ptr.data_mut() };
        child.parent = Some(parent_ptr.weak());
        if let Some(last_child_ptr) = &parent.last_child.as_ref().and_then(|x| x.rc()) {
            let last_child = unsafe { &mut *last_child_ptr.data_mut() };
            child.prev_sibling = Some(last_child_ptr.weak());
            last_child.next_sibling = Some(child_ptr.clone());
        } else {
            parent.first_child = Some(child_ptr.clone());
        }
        parent.last_child = Some(child_ptr.weak());
    }

    /// Insert a tree as the previous sibling node of the current node
    #[inline]
    pub fn insert(&mut self, target: &ForestNodeRc<T>) {
        let before_ptr = &self.inner;
        let before = unsafe { &mut *before_ptr.data_mut() };
        let parent_ptr = match before.parent.as_ref() {
            None => return,
            Some(x) => x,
        };
        self.check_insertion(parent_ptr, target);
        let parent_ptr = parent_ptr.rc();
        let mut parent = parent_ptr.as_ref().map(|x| unsafe { &mut *x.data_mut() });
        let child_ptr = &target.inner;
        let child = unsafe { &mut *child_ptr.data_mut() };
        child.parent = parent_ptr.as_ref().map(|x| x.weak());
        match before.prev_sibling.as_ref() {
            None => {
                if let Some(parent) = &mut parent {
                    parent.first_child = Some(child_ptr.clone());
                }
            }
            Some(prev_ptr) => {
                if let Some(prev_ptr) = prev_ptr.rc() {
                    let prev = unsafe { &mut *prev_ptr.data_mut() };
                    prev.next_sibling = Some(child_ptr.clone());
                }
            }
        }
        child.prev_sibling = before.prev_sibling.take();
        child.next_sibling = Some(before_ptr.clone());
        before.prev_sibling = Some(child_ptr.weak());
    }

    /// Remove the node from its parent node
    #[inline]
    pub fn detach(self) -> ForestNodeRc<T> {
        let child_ptr = self.inner.clone();
        let child = unsafe { &mut *child_ptr.data_mut() };
        let parent_ptr = child.parent.as_ref().and_then(|x| x.rc());
        let mut parent = parent_ptr.as_ref().map(|x| unsafe { &mut *x.data_mut() });
        let prev_ptr = child.prev_sibling.as_ref();
        let next_ptr = child.next_sibling.as_ref();
        match &next_ptr {
            None => {
                if let Some(parent) = &mut parent {
                    parent.last_child = prev_ptr.cloned();
                }
            }
            Some(next_ptr) => {
                let next = unsafe { &mut *next_ptr.data_mut() };
                next.prev_sibling = prev_ptr.cloned();
            }
        }
        match prev_ptr {
            None => {
                if let Some(parent) = &mut parent {
                    parent.first_child = next_ptr.cloned();
                }
            }
            Some(prev_ptr) => {
                if let Some(prev_ptr) = prev_ptr.rc() {
                    let prev = unsafe { &mut *prev_ptr.data_mut() };
                    prev.next_sibling = next_ptr.cloned();
                }
            }
        }
        child.parent = None;
        child.prev_sibling = None;
        child.next_sibling = None;
        ForestNodeRc { inner: child_ptr }
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
    #[inline]
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
    #[inline]
    pub fn as_ref<'b>(&'b self) -> ForestValue<'b, T> {
        ForestValue { v: self.v }
    }

    #[inline]
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

    struct DropTest {
        v: usize,
        dropped: bool,
    }

    impl Debug for DropTest {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.v)
        }
    }

    impl Drop for DropTest {
        fn drop(&mut self) {
            assert_eq!(self.dropped, false);
            self.dropped = true;
        }
    }

    impl From<usize> for DropTest {
        fn from(v: usize) -> Self {
            Self { v, dropped: false }
        }
    }

    fn check_pointers(tree: &ForestNode<DropTest>) {
        fn rec(node: &ForestNode<DropTest>) {
            let mut prev = None;
            let mut cur_option = node.first_child();
            while let Some(cur) = cur_option {
                assert!(cur.parent_rc().as_ref().unwrap().ptr_eq(&node.rc()));
                if let Some(prev) = prev.as_ref() {
                    assert!(cur.prev_sibling_rc().unwrap().ptr_eq(&prev));
                } else {
                    assert!(cur.prev_sibling_rc().is_none())
                }
                rec(&cur);
                assert_eq!(cur.dropped, false);
                prev = Some(cur.rc());
                cur_option = cur.next_sibling();
            }
            if let Some(prev) = prev.as_ref() {
                assert!(node.last_child_rc().unwrap().ptr_eq(&prev));
            } else {
                assert!(node.last_child_rc().is_none())
            }
        }
        assert!(tree.parent_rc().is_none());
        assert!(tree.next_sibling().is_none());
        assert!(tree.prev_sibling_rc().is_none());
        rec(&tree);
    }

    #[test]
    fn append() {
        let tree: ForestNodeRc<DropTest> = ForestNodeRc::new_forest(1.into());
        {
            let mut n1 = tree.borrow_mut();
            let n2 = n1.new_tree(2.into());
            {
                let mut n2 = n1.borrow_mut(&n2);
                let n3 = n2.new_tree(3.into());
                n2.append(&n3);
                let n4 = n2.new_tree(4.into());
                n2.append(&n4);
            }
            n1.append(&n2);
            assert_eq!(
                format!("{:?}", n1),
                r#"ForestNode(1) [ForestNode(2) [ForestNode(3) [], ForestNode(4) []]]"#
            );
        }
        check_pointers(&tree.borrow());
    }

    #[test]
    fn insert() {
        let tree: ForestNodeRc<DropTest> = ForestNodeRc::new_forest(1.into());
        {
            let mut n1 = tree.borrow_mut();
            let n2 = n1.new_tree(2.into());
            {
                let mut n2 = n1.borrow_mut(&n2);
                let n3 = n2.new_tree(3.into());
                n2.append(&n3);
                let n4 = n2.new_tree(4.into());
                n2.first_child_mut().unwrap().insert(&n4);
                let n5 = n2.new_tree(5.into());
                n2.first_child_mut()
                    .unwrap()
                    .next_sibling_mut()
                    .unwrap()
                    .insert(&n5);
            }
            n1.append(&n2);
            assert_eq!(
                format!("{:?}", n1),
                r#"ForestNode(1) [ForestNode(2) [ForestNode(4) [], ForestNode(5) [], ForestNode(3) []]]"#
            );
        }
        check_pointers(&tree.borrow());
    }

    #[test]
    fn detach() {
        let tree: ForestNodeRc<DropTest> = ForestNodeRc::new_forest(1.into());
        {
            let mut n1 = tree.borrow_mut();
            let n2 = n1.new_tree(2.into());
            {
                let mut n2 = n1.borrow_mut(&n2);
                let n3 = n2.new_tree(3.into());
                n2.append(&n3);
                let n4 = n2.new_tree(4.into());
                n2.append(&n4);
                let n5 = n2.new_tree(5.into());
                n2.append(&n5);
            }
            n1.append(&n2);
            assert_eq!(
                format!("{:?}", n1),
                r#"ForestNode(1) [ForestNode(2) [ForestNode(3) [], ForestNode(4) [], ForestNode(5) []]]"#
            );
            {
                let n4 = {
                    let mut n2 = n1.first_child_mut().unwrap();
                    let mut n3 = n2.first_child_mut().unwrap();
                    let n4 = n3.next_sibling_mut().unwrap();
                    n4.detach()
                };
                assert_eq!(format!("{:?}", n1.borrow_mut(&n4)), r#"ForestNode(4) []"#);
            }
            assert_eq!(
                format!("{:?}", n1),
                r#"ForestNode(1) [ForestNode(2) [ForestNode(3) [], ForestNode(5) []]]"#
            );
            {
                let n2 = {
                    let n2 = n1.first_child_mut().unwrap();
                    n2.detach()
                };
                assert_eq!(
                    format!("{:?}", n1.borrow_mut(&n2)),
                    r#"ForestNode(2) [ForestNode(3) [], ForestNode(5) []]"#
                );
            }
            assert_eq!(format!("{:?}", n1), r#"ForestNode(1) []"#);
        }
        check_pointers(&tree.borrow());
    }
}
