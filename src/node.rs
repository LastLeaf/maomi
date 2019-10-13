use std::ops::{Deref, DerefMut};
use std::rc::{Rc, Weak};
use std::fmt;
use std::any::Any;
use std::ops::Range;
use me_cell::*;

use super::{Component, ComponentTemplate, backend::Backend};

const COMPONENT_TAG_NAME: &'static str = "maomi-component"; // FIXME use a friendly name

pub(crate) fn create_component<'a, B: Backend, T: ElementRefMut<'a, B>, C: 'static + Component>(n: &mut T, component: Box<C>, children: Vec<NodeRc<B>>) -> ComponentNodeRc<B> {
    let backend = n.backend().clone();
    let shadow_root = VirtualNodeRc {
        c: Rc::new(n.as_me_ref_mut_handle().entrance(VirtualNode { backend, tag_name: "shadow-root", property: VirtualNodeProperty::None, children: vec![] }))
    };
    let backend = n.backend().clone();
    let backend_element = backend.create_element(COMPONENT_TAG_NAME);
    let ret = ComponentNodeRc {
        c: Rc::new(n.as_me_ref_mut_handle().entrance(ComponentNode { backend, backend_element, component, shadow_root: shadow_root.clone(), children }))
    };
    {
        let mut component_node = unsafe { ret.borrow_mut_unsafe_with(n) };
        let shadow_root_content = <C as ComponentTemplate>::template(&mut component_node, false);
        let mut shadow_root = shadow_root.borrow_mut_with(n);
        if let Some(mut shadow_root_content) = shadow_root_content {
            std::mem::swap(&mut shadow_root.children, &mut shadow_root_content);
        }
    }
    ret
}

pub struct NativeNode<B: Backend> {
    pub(crate) backend: Rc<B>,
    pub(crate) backend_element: B::BackendElement,
    pub(crate) tag_name: &'static str,
    pub(crate) attributes: Vec<(&'static str, String)>,
    pub(crate) children: Vec<NodeRc<B>>,
    // pub(crate) owner: Option<ComponentNodeWeak<B>>,
    // pub(crate) parent: Option<ComponentNodeWeak<B>>,
}
impl<B: Backend> NativeNode<B> {
    pub fn tag_name(&self) -> &str {
        &self.tag_name
    }
    pub fn get_attribute(&self, name: &'static str) -> Option<&str> {
        self.attributes.iter().find(|x| x.0 == name).map(|x| x.1.as_str())
    }
    pub fn set_attribute<T: ToString>(&mut self, name: &'static str, value: T) {
        match self.attributes.iter_mut().find(|x| x.0 == name) {
            Some(x) => {
                x.1 = value.to_string();
                return
            },
            None => { }
        }
        self.attributes.push((name, value.to_string()))
    }
}
impl<'a, B: Backend> NativeNodeRef<'a, B> {
    pub fn backend_element(&self) -> &B::BackendElement {
        &self.backend_element
    }
    pub fn children(&self) -> &Vec<NodeRc<B>> {
        &self.children
    }
    fn debug_fmt(&self, f: &mut fmt::Formatter<'_>, level: u32) -> fmt::Result {
        for _ in 0..level {
            write!(f, "  ")?;
        }
        let n: &NativeNode<B> = &**self;
        writeln!(f, "{:?}", n)?;
        for child in self.children.iter() {
            child.borrow_with(self).debug_fmt(f, level + 1)?;
        }
        Ok(())
    }
}
impl<'a, B: Backend> NativeNodeRefMut<'a, B> {
    pub fn backend_element(&self) -> &B::BackendElement {
        &self.backend_element
    }
    pub fn children(&self) -> &Vec<NodeRc<B>> {
        &self.children
    }
}
impl<B: Backend> fmt::Debug for NativeNode<B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<{}", self.tag_name)?;
        for (name, value) in self.attributes.iter() {
            write!(f, r#" {}="{}""#, name, value)?;
        }
        write!(f, ">")?;
        Ok(())
    }
}

pub enum VirtualNodeProperty {
    None,
    Slot(&'static str),
    Branch(usize),
    List(Box<dyn Any>),
}

pub struct VirtualNode<B: Backend> {
    pub(crate) backend: Rc<B>,
    pub(crate) tag_name: &'static str,
    pub(crate) property: VirtualNodeProperty,
    pub(crate) children: Vec<NodeRc<B>>,
}
impl<B: Backend> VirtualNode<B> {
    pub fn tag_name(&self) -> &str {
        &self.tag_name
    }
    pub fn property(&self) -> &VirtualNodeProperty {
        &self.property
    }
    pub fn set_property(&mut self, property: VirtualNodeProperty) {
        self.property = property;
    }
}
impl<'a, B: Backend> VirtualNodeRef<'a, B> {
    pub fn children(&self) -> &Vec<NodeRc<B>> {
        &self.children
    }
    fn debug_fmt(&self, f: &mut fmt::Formatter<'_>, level: u32) -> fmt::Result {
        for _ in 0..level {
            write!(f, "  ")?;
        }
        let n: &VirtualNode<B> = &**self;
        writeln!(f, "{:?}", n)?;
        for child in self.children.iter() {
            child.borrow_with(self).debug_fmt(f, level + 1)?;
        }
        Ok(())
    }
}
impl<'a, B: Backend> VirtualNodeRefMut<'a, B> {
    pub fn property_mut(&mut self) -> &mut VirtualNodeProperty {
        &mut self.property
    }
    pub fn children(&self) -> &Vec<NodeRc<B>> {
        &self.children
    }
    pub fn replace_children_list(&mut self, mut list: Vec<NodeRc<B>>) {
        std::mem::swap(&mut self.children, &mut list);
    }
    pub fn remove_range(&mut self, r: Range<usize>) -> Vec<NodeRc<B>> {
        self.children.splice(r, vec![]).collect()
    }
    pub fn insert_list(&mut self, pos: usize, list: Vec<NodeRc<B>>) {
        let _: Vec<_> = self.children.splice(pos..pos, list).collect();
    }
}
impl<B: Backend> fmt::Debug for VirtualNode<B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({})", self.tag_name)
    }
}

pub struct ComponentNode<B: Backend> {
    pub(crate) backend: Rc<B>,
    pub(crate) backend_element: B::BackendElement,
    pub(crate) component: Box<dyn Component>,
    pub(crate) shadow_root: VirtualNodeRc<B>,
    pub(crate) children: Vec<NodeRc<B>>,
}
impl<'a, B: 'static + Backend> ComponentNode<B> {
    pub fn shadow_root_rc(&self) -> &VirtualNodeRc<B> {
        &self.shadow_root
    }
    pub fn as_component<C: 'static + Component>(&self) -> &C {
        let c: &dyn Any = &self.component;
        c.downcast_ref().unwrap()
    }
    pub fn as_component_mut<C: 'static + Component>(&mut self) -> &mut C {
        let c: &mut dyn Any = &mut self.component;
        c.downcast_mut().unwrap()
    }
    pub fn try_as_component<C: 'static + Component>(&self) -> Option<&C> {
        let c: &dyn Any = &self.component;
        c.downcast_ref()
    }
    pub fn try_as_component_mut<C: 'static + Component>(&mut self) -> Option<&mut C> {
        let c: &mut dyn Any = &mut self.component;
        c.downcast_mut()
    }
}
impl<'a, B: Backend> ComponentNodeRef<'a, B> {
    pub fn backend_element(&self) -> &B::BackendElement {
        &self.backend_element
    }
    pub fn shadow_root(&self) -> VirtualNodeRef<B> {
        self.shadow_root.borrow_with(self)
    }
    pub fn children(&self) -> &Vec<NodeRc<B>> {
        &self.children
    }
    fn debug_fmt(&self, f: &mut fmt::Formatter<'_>, level: u32) -> fmt::Result {
        for _ in 0..level {
            write!(f, "  ")?;
        }
        let n: &ComponentNode<B> = &**self;
        writeln!(f, "{:?}", n)?;
        n.shadow_root.borrow_with(self).debug_fmt(f, level + 1)?;
        for child in self.children.iter() {
            child.borrow_with(self).debug_fmt(f, level + 1)?;
        }
        Ok(())
    }
}
impl<'a, B: Backend> ComponentNodeRefMut<'a, B> {
    pub fn backend_element(&self) -> &B::BackendElement {
        &self.backend_element
    }
    pub fn children(&self) -> &Vec<NodeRc<B>> {
        &self.children
    }
    pub fn new_native_node(&mut self, tag_name: &'static str, attributes: Vec<(&'static str, String)>, children: Vec<NodeRc<B>>) -> NativeNodeRc<B> {
        let backend = self.backend().clone();
        let backend_element = backend.create_element(tag_name);
        NativeNodeRc {
            c: Rc::new(self.as_me_ref_mut_handle().entrance(NativeNode { backend, backend_element, tag_name, attributes, children }))
        }
    }
    pub fn new_virtual_node(&mut self, tag_name: &'static str, property: VirtualNodeProperty, children: Vec<NodeRc<B>>) -> VirtualNodeRc<B> {
        let backend = self.backend().clone();
        VirtualNodeRc {
            c: Rc::new(self.as_me_ref_mut_handle().entrance(VirtualNode { backend, tag_name, property, children }))
        }
    }
    pub fn new_component_node<C: 'static + Component>(&mut self, component: Box<C>, children: Vec<NodeRc<B>>) -> ComponentNodeRc<B> {
        create_component(self, component, children)
    }
    pub fn new_text_node(&mut self, text_content: String) -> TextNodeRc<B> {
        let backend = self.backend().clone();
        let backend_element = backend.create_text_node(text_content.as_str());
        TextNodeRc {
            c: Rc::new(self.as_me_ref_mut_handle().entrance(TextNode { backend, backend_element, text_content }))
        }
    }
}
impl<B: Backend> fmt::Debug for ComponentNode<B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<Component>")
    }
}
impl<'a, B: Backend> Deref for ComponentNode<B> {
    type Target = Box<dyn Component>;
    fn deref(&self) -> &Box<dyn Component> {
        &self.component
    }
}
impl<B: Backend> DerefMut for ComponentNode<B> {
    fn deref_mut(&mut self) -> &mut Box<dyn Component> {
        &mut self.component
    }
}

pub struct TextNode<B: Backend> {
    pub(crate) backend: Rc<B>,
    pub(crate) backend_element: B::BackendTextNode,
    pub(crate) text_content: String,
}
impl<B: Backend> TextNode<B> {
    pub fn text_content(&self) -> &str {
        &self.text_content
    }
    pub fn set_text_content<T: Into<String>>(&mut self, c: T) {
        self.text_content = c.into();
    }
}
impl<'a, B: Backend> TextNodeRef<'a, B> {
    pub fn backend_element(&self) -> &B::BackendTextNode {
        &self.backend_element
    }
    fn debug_fmt(&self, f: &mut fmt::Formatter<'_>, level: u32) -> fmt::Result {
        for _ in 0..level {
            write!(f, "  ")?;
        }
        let n: &TextNode<B> = &**self;
        writeln!(f, "{:?}", n)?;
        Ok(())
    }
}
impl<'a, B: Backend> TextNodeRefMut<'a, B> {
    pub fn backend_element(&self) -> &B::BackendTextNode {
        &self.backend_element
    }
}
impl<B: Backend> fmt::Debug for TextNode<B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{:?}", self.text_content)
    }
}

pub enum NodeRc<B: Backend> {
    NativeNode(NativeNodeRc<B>),
    VirtualNode(VirtualNodeRc<B>),
    ComponentNode(ComponentNodeRc<B>),
    TextNode(TextNodeRc<B>),
}
impl<B: Backend> Clone for NodeRc<B> {
    fn clone(&self) -> Self {
        match self {
            NodeRc::NativeNode(x) => NodeRc::NativeNode(x.clone()),
            NodeRc::VirtualNode(x) => NodeRc::VirtualNode(x.clone()),
            NodeRc::ComponentNode(x) => NodeRc::ComponentNode(x.clone()),
            NodeRc::TextNode(x) => NodeRc::TextNode(x.clone()),
        }
    }
}
impl<B: Backend> NodeRc<B> {
    pub fn borrow<'a>(&'a self) -> NodeRef<'a, B> {
        match self {
            NodeRc::NativeNode(x) => NodeRef::NativeNode(x.borrow()),
            NodeRc::VirtualNode(x) => NodeRef::VirtualNode(x.borrow()),
            NodeRc::ComponentNode(x) => NodeRef::ComponentNode(x.borrow()),
            NodeRc::TextNode(x) => NodeRef::TextNode(x.borrow()),
        }
    }
    pub fn borrow_mut<'a>(&'a self) -> NodeRefMut<'a, B> {
        match self {
            NodeRc::NativeNode(x) => NodeRefMut::NativeNode(x.borrow_mut()),
            NodeRc::VirtualNode(x) => NodeRefMut::VirtualNode(x.borrow_mut()),
            NodeRc::ComponentNode(x) => NodeRefMut::ComponentNode(x.borrow_mut()),
            NodeRc::TextNode(x) => NodeRefMut::TextNode(x.borrow_mut()),
        }
    }
    pub fn borrow_with<'a: 'b, 'b, U>(&'b self, source: &'a U) -> NodeRef<'b, B> where U: ElementRef<'a, B> {
        match self {
            NodeRc::NativeNode(x) => NodeRef::NativeNode(x.borrow_with(source)),
            NodeRc::VirtualNode(x) => NodeRef::VirtualNode(x.borrow_with(source)),
            NodeRc::ComponentNode(x) => NodeRef::ComponentNode(x.borrow_with(source)),
            NodeRc::TextNode(x) => NodeRef::TextNode(x.borrow_with(source)),
        }
    }
    pub fn borrow_mut_with<'a: 'b, 'b, U>(&'b self, source: &'a mut U) -> NodeRefMut<'b, B> where U: ElementRefMut<'a, B> {
        match self {
            NodeRc::NativeNode(x) => NodeRefMut::NativeNode(x.borrow_mut_with(source)),
            NodeRc::VirtualNode(x) => NodeRefMut::VirtualNode(x.borrow_mut_with(source)),
            NodeRc::ComponentNode(x) => NodeRefMut::ComponentNode(x.borrow_mut_with(source)),
            NodeRc::TextNode(x) => NodeRefMut::TextNode(x.borrow_mut_with(source)),
        }
    }
    pub unsafe fn borrow_mut_unsafe_with<'a: 'b, 'b, U>(&'b self, source: &'a mut U) -> NodeRefMut<'b, B> where U: ElementRefMut<'a, B> {
        match self {
            NodeRc::NativeNode(x) => NodeRefMut::NativeNode(x.borrow_mut_unsafe_with(source)),
            NodeRc::VirtualNode(x) => NodeRefMut::VirtualNode(x.borrow_mut_unsafe_with(source)),
            NodeRc::ComponentNode(x) => NodeRefMut::ComponentNode(x.borrow_mut_unsafe_with(source)),
            NodeRc::TextNode(x) => NodeRefMut::TextNode(x.borrow_mut_unsafe_with(source)),
        }
    }
}

#[derive(Clone)]
pub enum NodeWeak<B: Backend> {
    NativeNode(NativeNodeWeak<B>),
    VirtualNode(VirtualNodeWeak<B>),
    ComponentNode(ComponentNodeWeak<B>),
    TextNode(TextNodeWeak<B>),
}
impl<B: Backend> NodeWeak<B> {
    pub fn upgrade(&self) -> Option<NodeRc<B>> {
        match self {
            NodeWeak::NativeNode(x) => x.upgrade().map(|x| NodeRc::NativeNode(x)),
            NodeWeak::VirtualNode(x) => x.upgrade().map(|x| NodeRc::VirtualNode(x)),
            NodeWeak::ComponentNode(x) => x.upgrade().map(|x| NodeRc::ComponentNode(x)),
            NodeWeak::TextNode(x) => x.upgrade().map(|x| NodeRc::TextNode(x)),
        }
    }
}

pub enum NodeRef<'a, B: Backend> {
    NativeNode(NativeNodeRef<'a, B>),
    VirtualNode(VirtualNodeRef<'a, B>),
    ComponentNode(ComponentNodeRef<'a, B>),
    TextNode(TextNodeRef<'a, B>),
}
pub enum NodeRefMut<'a, B: Backend> {
    NativeNode(NativeNodeRefMut<'a, B>),
    VirtualNode(VirtualNodeRefMut<'a, B>),
    ComponentNode(ComponentNodeRefMut<'a, B>),
    TextNode(TextNodeRefMut<'a, B>),
}
impl<'a, B: Backend> NodeRef<'a, B> {
    fn debug_fmt(&self, f: &mut fmt::Formatter<'_>, level: u32) -> fmt::Result {
        match self {
            Self::NativeNode(x) => x.debug_fmt(f, level),
            Self::VirtualNode(x) => x.debug_fmt(f, level),
            Self::ComponentNode(x) => x.debug_fmt(f, level),
            Self::TextNode(x) => x.debug_fmt(f, level),
        }
    }
}
impl<'a, B: Backend> fmt::Debug for NodeRef<'a, B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.debug_fmt(f, 0)
    }
}
impl<'a, B: Backend> NodeRefMut<'a, B> {
    pub fn to_ref<'b>(&'b self) -> NodeRef<'b, B> where 'a: 'b {
        match self {
            Self::NativeNode(x) => NodeRef::NativeNode(x.to_ref()),
            Self::VirtualNode(x) => NodeRef::VirtualNode(x.to_ref()),
            Self::ComponentNode(x) => NodeRef::ComponentNode(x.to_ref()),
            Self::TextNode(x) => NodeRef::TextNode(x.to_ref()),
        }
    }
    fn debug_fmt(&self, f: &mut fmt::Formatter<'_>, level: u32) -> fmt::Result {
        match self {
            Self::NativeNode(x) => x.debug_fmt(f, level),
            Self::VirtualNode(x) => x.debug_fmt(f, level),
            Self::ComponentNode(x) => x.debug_fmt(f, level),
            Self::TextNode(x) => x.debug_fmt(f, level),
        }
    }
}
impl<'a, B: Backend> fmt::Debug for NodeRefMut<'a, B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.debug_fmt(f, 0)
    }
}

pub type TemplateNodeFn<B, T> = Box<dyn Fn(&T) -> NodeRc<B>>;

pub trait ElementRef<'a, B: Backend> {
    fn backend(&self) -> &Rc<B>;
    fn as_me_ref_handle(&self) -> &MeRefHandle<'a>;
    fn as_node_ref<'b>(self) -> NodeRef<'b, B> where 'a: 'b;
}
pub trait ElementRefMut<'a, B: Backend> {
    fn backend(&self) -> &Rc<B>;
    fn as_me_ref_mut_handle<'b>(&'b mut self) -> &'b mut MeRefMutHandle<'a> where 'a: 'b;
    fn as_node_ref_mut<'b>(self) -> NodeRefMut<'b, B> where 'a: 'b;
}

impl<'a, B: Backend> ElementRef<'a, B> for NodeRef<'a, B> {
    fn backend(&self) -> &Rc<B> {
        match self {
            Self::NativeNode(x) => &x.backend,
            Self::VirtualNode(x) => &x.backend,
            Self::ComponentNode(x) => &x.backend,
            Self::TextNode(x) => &x.backend,
        }
    }
    fn as_me_ref_handle(&self) -> &MeRefHandle<'a> {
        match self {
            Self::NativeNode(x) => x.as_me_ref_handle(),
            Self::VirtualNode(x) => x.as_me_ref_handle(),
            Self::ComponentNode(x) => x.as_me_ref_handle(),
            Self::TextNode(x) => x.as_me_ref_handle(),
        }
    }
    fn as_node_ref<'b>(self) -> NodeRef<'b, B> where 'a: 'b {
        match self {
            Self::NativeNode(x) => x.as_node_ref(),
            Self::VirtualNode(x) => x.as_node_ref(),
            Self::ComponentNode(x) => x.as_node_ref(),
            Self::TextNode(x) => x.as_node_ref(),
        }
    }
}
impl<'a, B: Backend> ElementRefMut<'a, B> for NodeRefMut<'a, B> {
    fn backend(&self) -> &Rc<B> {
        match self {
            Self::NativeNode(x) => &x.backend,
            Self::VirtualNode(x) => &x.backend,
            Self::ComponentNode(x) => &x.backend,
            Self::TextNode(x) => &x.backend,
        }
    }
    fn as_me_ref_mut_handle<'b>(&'b mut self) -> &'b mut MeRefMutHandle<'a> where 'a: 'b {
        match self {
            Self::NativeNode(x) => x.as_me_ref_mut_handle(),
            Self::VirtualNode(x) => x.as_me_ref_mut_handle(),
            Self::ComponentNode(x) => x.as_me_ref_mut_handle(),
            Self::TextNode(x) => x.as_me_ref_mut_handle(),
        }
    }
    fn as_node_ref_mut<'b>(self) -> NodeRefMut<'b, B> where 'a: 'b {
        match self {
            Self::NativeNode(x) => x.as_node_ref_mut(),
            Self::VirtualNode(x) => x.as_node_ref_mut(),
            Self::ComponentNode(x) => x.as_node_ref_mut(),
            Self::TextNode(x) => x.as_node_ref_mut(),
        }
    }
}

macro_rules! some_node_def {
    ($t: ident, $rc: ident, $weak: ident, $r: ident, $rm: ident) => {
        pub struct $rc<B: Backend> {
            c: Rc<MeCell<$t<B>>>
        }
        impl<B: Backend> $rc<B> {
            #[allow(dead_code)]
            pub(crate) fn new_with_me_cell_group(c: $t<B>) -> Self {
                Self {
                    c: Rc::new(MeCell::new_group(c))
                }
            }
            pub fn borrow<'a>(&'a self) -> $r<'a, B> {
                $r { c: self.c.borrow() }
            }
            pub fn borrow_mut<'a>(&'a self) -> $rm<'a, B> {
                $rm { c: self.c.borrow_mut() }
            }
            pub fn borrow_with<'a: 'b, 'b, U>(&'b self, source: &'b U) -> $r<'b, B> where U: ElementRef<'a, B> {
                $r { c: self.c.borrow_with_handle(source.as_me_ref_handle()) }
            }
            pub fn borrow_mut_with<'a: 'b, 'b, U>(&'b self, source: &'b mut U) -> $rm<'b, B> where U: ElementRefMut<'a, B> {
                $rm { c: self.c.borrow_mut_with_handle(source.as_me_ref_mut_handle()) }
            }
            pub unsafe fn borrow_mut_unsafe_with<'a: 'b, 'b, 'c, U>(&'c self, source: &'b mut U) -> $rm<'c, B> where U: ElementRefMut<'a, B> {
                $rm { c: self.c.borrow_mut_unsafe_with_handle(source.as_me_ref_mut_handle()) }
            }
            pub fn downgrade(&self) -> $weak<B> {
                $weak { c: Rc::downgrade(&self.c) }
            }
        }
        impl<B: Backend> Clone for $rc<B> {
            fn clone(&self) -> Self {
                Self { c: self.c.clone() }
            }
        }
        impl<B: Backend> From<$rc<B>> for NodeRc<B> {
            fn from(s: $rc<B>) -> Self {
                NodeRc::$t(s)
            }
        }

        pub struct $weak<B: Backend> {
            c: Weak<MeCell<$t<B>>>
        }
        impl<B: Backend> $weak<B> {
            pub fn upgrade(&self) -> Option<$rc<B>> {
                self.c.upgrade().map(|x| {
                    $rc { c: x }
                })
            }
        }
        impl<B: Backend> Clone for $weak<B> {
            fn clone(&self) -> Self {
                Self { c: self.c.clone() }
            }
        }
        impl<B: Backend> From<$weak<B>> for NodeWeak<B> {
            fn from(s: $weak<B>) -> Self {
                NodeWeak::$t(s)
            }
        }

        pub struct $r<'a, B: Backend> {
            c: MeRef<'a, $t<B>>
        }
        impl<'a, B: Backend> $r<'a, B> { }
        impl<'a, B: Backend> Deref for $r<'a, B> {
            type Target = $t<B>;
            fn deref(&self) -> &$t<B> {
                &*self.c
            }
        }
        impl<'a, B: Backend> fmt::Debug for $r<'a, B> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.debug_fmt(f, 0)
            }
        }
        impl<'a, B: Backend> ElementRef<'a, B> for $r<'a, B> {
            fn backend(&self) -> &Rc<B> {
                &self.backend
            }
            fn as_me_ref_handle(&self) -> &MeRefHandle<'a> {
                self.c.handle()
            }
            fn as_node_ref<'b>(self) -> NodeRef<'b, B> where 'a: 'b {
                NodeRef::$t($r { c: self.c.map(|x| x) })
            }
        }
        impl<'a, B: Backend> From<$r<'a, B>> for NodeRef<'a, B> {
            fn from(s: $r<'a, B>) -> Self {
                NodeRef::$t(s)
            }
        }

        pub struct $rm<'a, B: Backend> {
            c: MeRefMut<'a, $t<B>>
        }
        impl<'a, B: Backend> $rm<'a, B> {
            pub fn to_ref<'b>(&'b self) -> $r<'b, B> where 'a: 'b {
                $r { c: self.c.to_ref() }
            }
            fn debug_fmt(&self, f: &mut fmt::Formatter<'_>, level: u32) -> fmt::Result {
                self.to_ref().debug_fmt(f, level)
            }
        }
        impl<'a, B: Backend> Deref for $rm<'a, B> {
            type Target = $t<B>;
            fn deref(&self) -> &$t<B> {
                &*self.c
            }
        }
        impl<'a, B: Backend> DerefMut for $rm<'a, B> {
            fn deref_mut(&mut self) -> &mut $t<B> {
                &mut *self.c
            }
        }
        impl<'a, B: Backend> fmt::Debug for $rm<'a, B> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.debug_fmt(f, 0)
            }
        }
        impl<'a, B: Backend> ElementRefMut<'a, B> for $rm<'a, B> {
            fn backend(&self) -> &Rc<B> {
                &self.backend
            }
            fn as_me_ref_mut_handle<'b>(&'b mut self) -> &'b mut MeRefMutHandle<'a> where 'a: 'b {
                self.c.handle_mut()
            }
            fn as_node_ref_mut<'b>(self) -> NodeRefMut<'b, B> where 'a: 'b {
                NodeRefMut::$t($rm { c: self.c.map(|x| x) })
            }
        }
        impl<'a, B: Backend> From<$rm<'a, B>> for NodeRefMut<'a, B> {
            fn from(s: $rm<'a, B>) -> Self {
                NodeRefMut::$t(s)
            }
        }
    }
}
some_node_def!(NativeNode, NativeNodeRc, NativeNodeWeak, NativeNodeRef, NativeNodeRefMut);
some_node_def!(VirtualNode, VirtualNodeRc, VirtualNodeWeak, VirtualNodeRef, VirtualNodeRefMut);
some_node_def!(ComponentNode, ComponentNodeRc, ComponentNodeWeak, ComponentNodeRef, ComponentNodeRefMut);
some_node_def!(TextNode, TextNodeRc, TextNodeWeak, TextNodeRef, TextNodeRefMut);
