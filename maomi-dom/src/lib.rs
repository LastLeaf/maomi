use maomi::{
    backend::{tree::*, *},
    error::Error,
};
use wasm_bindgen::{JsCast, JsValue};

#[cfg(all(not(feature = "prerendering"), not(feature = "prerendering-apply")))]
macro_rules! dom_state_ty {
    ($t:ty, $u:ty, $v:ty) => {
        DomState<$t>
    };
}

#[cfg(all(not(feature = "prerendering"), feature = "prerendering-apply"))]
macro_rules! dom_state_ty {
    ($t:ty, $u:ty, $v:ty) => {
        DomState<$t, $v>
    };
}

#[cfg(all(feature = "prerendering", not(feature = "prerendering-apply")))]
macro_rules! dom_state_ty {
    ($t:ty, $u:ty, $v:ty) => {
        DomState<$t, $u>
    };
}

#[cfg(all(feature = "prerendering", feature = "prerendering-apply"))]
macro_rules! dom_state_ty {
    ($t:ty, $u:ty, $v:ty) => {
        DomState<$t, $u, $v>
    };
}

pub mod base_element;
use base_element::DomElement;
#[cfg(feature = "prerendering")]
use base_element::PrerenderingElement;
#[cfg(feature = "prerendering-apply")]
use base_element::RematchedDomElem;
pub mod element;
pub mod virtual_element;
use virtual_element::DomVirtualElement;
pub mod text_node;
use text_node::DomTextNode;
pub mod class_list;
mod composing;
pub mod event;
use event::DomListeners;

pub mod prelude {
    pub use crate::DomBackend;
    pub use maomi_dom_macro::dom_css;
}

thread_local! {
    pub(crate) static WINDOW: web_sys::Window = web_sys::window().expect("Cannot init DOM backend outside web page environment");
    pub(crate) static DOCUMENT: web_sys::Document = {
        WINDOW.with(|window| {
            window.document().expect("Cannot init DOM backend when document is not ready")
        })
    };
}

fn log_js_error(err: &JsValue) {
    if let Some(err) = err.dyn_ref::<js_sys::Error>() {
        log::error!("{}", err.message());
    } else {
        log::error!("(JavaScript Error)");
    }
}

/// A common async runner for DOM environment
#[inline]
pub fn async_task(fut: impl 'static + std::future::Future<Output = ()>) {
    wasm_bindgen_futures::spawn_local(fut);
}

#[cfg(all(not(feature = "prerendering"), not(feature = "prerendering-apply")))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum DomState<T> {
    Normal(T),
}

#[cfg(all(not(feature = "prerendering"), feature = "prerendering-apply"))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum DomState<T, V> {
    Normal(T),
    PrerenderingApply(V),
}

#[cfg(all(feature = "prerendering", not(feature = "prerendering-apply")))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum DomState<T, U> {
    Normal(T),
    Prerendering(U),
}

#[cfg(all(feature = "prerendering", feature = "prerendering-apply"))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum DomState<T, U, V> {
    Normal(T),
    Prerendering(U),
    PrerenderingApply(V),
}

#[derive(Debug, Default)]
pub(crate) struct WriteHtmlState {
    #[allow(dead_code)]
    prev_is_text_node: bool,
}

/// A DOM backend
pub struct DomBackend {
    backend_stage: BackendStage,
    tree: tree::ForestNodeRc<DomGeneralElement>,
    #[allow(dead_code)]
    listeners: dom_state_ty!(DomListeners, (), ()),
}

impl DomBackend {
    /// Create a backend that rendering under the specified DOM element
    #[inline]
    pub fn new_with_element(dom_elem: web_sys::Element) -> Result<Self, Error> {
        Ok(Self::wrap_root_element(dom_elem)?)
    }

    /// Create a backend that rendering under the DOM element with the `id`
    #[inline]
    pub fn new_with_element_id(id: &str) -> Result<Self, Error> {
        let dom_elem = DOCUMENT
            .with(|document| document.get_element_by_id(id))
            .ok_or_else(|| Error::BackendError {
                msg: format!("Cannot find the element {:?}", id),
                err: None,
            })?;
        Ok(Self::wrap_root_element(dom_elem)?)
    }

    /// Create a backend that rendering under the DOM `<body>`
    #[inline]
    pub fn new_with_document_body() -> Result<Self, Error> {
        let dom_elem =
            DOCUMENT
                .with(|document| document.body())
                .ok_or_else(|| Error::BackendError {
                    msg: "Cannot find the <body> element".into(),
                    err: None,
                })?;
        Ok(Self::wrap_root_element(dom_elem.into())?)
    }

    fn wrap_root_element(dom_elem: web_sys::Element) -> Result<Self, Error> {
        let listeners = DomState::Normal(event::DomListeners::new(&dom_elem));
        let tree_root = {
            let ret = tree::ForestNodeRc::new_forest(DomGeneralElement::Element(unsafe {
                DomElement::new(DomState::Normal(dom_elem))
            }));
            let token = ret.token();
            if let DomGeneralElement::Element(x) = &mut *ret.borrow_mut() {
                x.init(token);
            } else {
                unreachable!()
            }
            ret
        };
        Ok(Self {
            backend_stage: BackendStage::Normal,
            tree: tree_root,
            listeners,
        })
    }

    /// Create a backend for prerendering
    ///
    /// The prerendering can generate HTML segment without DOM environment.
    /// It can be used for server side rendering.
    #[cfg(feature = "prerendering")]
    #[inline]
    pub fn prerendering() -> Self {
        let tree_root = {
            let ret = tree::ForestNodeRc::new_forest(DomGeneralElement::Element(unsafe {
                DomElement::new(DomState::Prerendering(PrerenderingElement::new("maomi")))
            }));
            let token = ret.token();
            if let DomGeneralElement::Element(x) = &mut *ret.borrow_mut() {
                x.init(token);
            } else {
                unreachable!()
            }
            ret
        };
        Self {
            backend_stage: BackendStage::Prerendering,
            tree: tree_root,
            listeners: DomState::Prerendering(()),
        }
    }

    /// Write the prerendering result to a `Write`
    ///
    /// The prerendering result is an HTML segment,
    /// which should be placed into the full HTML file as the final server response.
    #[cfg(feature = "prerendering")]
    #[inline]
    pub fn write_prerendering_html(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        let mut state = Default::default();
        DomGeneralElement::write_inner_html(&self.root(), w, &mut state)
    }

    /// Prepare a backend for using the prerendering result
    ///
    /// The prerendering result can be attached later with one of the `apply_prerendered_*` method.
    #[cfg(feature = "prerendering-apply")]
    #[inline]
    pub fn new_prerendered() -> Self {
        let tree_root = {
            let ret = tree::ForestNodeRc::new_forest(DomGeneralElement::Element(unsafe {
                DomElement::new(DomState::PrerenderingApply(
                    base_element::RematchedDomElem::new(),
                ))
            }));
            let token = ret.token();
            if let DomGeneralElement::Element(x) = &mut *ret.borrow_mut() {
                x.init(token);
            } else {
                unreachable!()
            }
            ret
        };
        Self {
            backend_stage: BackendStage::PrerenderingApply,
            tree: tree_root,
            listeners: DomState::PrerenderingApply(()),
        }
    }

    /// Attach the prerendering result with the specified DOM element
    #[cfg(feature = "prerendering-apply")]
    pub fn apply_prerendered_element(&mut self, dom_elem: web_sys::Element) -> Result<(), Error> {
        self.apply_prerendered(dom_elem.into())
    }

    /// Attach the prerendering result with the DOM element with the `id`
    #[cfg(feature = "prerendering-apply")]
    pub fn apply_prerendered_element_id(&mut self, id: &str) -> Result<(), Error> {
        let dom_elem = DOCUMENT
            .with(|document| document.get_element_by_id(id))
            .ok_or_else(|| Error::BackendError {
                msg: format!("Cannot find the element {:?}", id),
                err: None,
            })?;
        self.apply_prerendered(dom_elem.into())
    }

    /// Attach the prerendering result with the DOM `<body>`
    #[cfg(feature = "prerendering-apply")]
    pub fn apply_prerendered_document_body(&mut self) -> Result<(), Error> {
        let dom_elem =
            DOCUMENT
                .with(|document| document.body())
                .ok_or_else(|| Error::BackendError {
                    msg: "Cannot find the <body> element".into(),
                    err: None,
                })?;
        self.apply_prerendered(dom_elem.into())
    }

    #[cfg(feature = "prerendering-apply")]
    fn apply_prerendered(&mut self, dom_elem: web_sys::Element) -> Result<(), Error> {
        if self.backend_stage != BackendStage::PrerenderingApply {
            panic!("The backend is not in prerendering-apply stage");
        }
        self.backend_stage = BackendStage::Normal;
        self.listeners = DomState::Normal(event::DomListeners::new(&dom_elem));
        fn rematch_dom<'a>(
            n: &mut ForestNodeMut<'a, DomGeneralElement>,
            next_dom_elem: Option<web_sys::Node>,
            state: &mut WriteHtmlState,
        ) -> Result<Option<web_sys::Node>, Error> {
            enum ChildMatchKind {
                Virtual(Option<web_sys::Node>),
                NonVirtual(web_sys::Node),
            }
            let child_match_kind = {
                let ge: &mut DomGeneralElement = n;
                match ge {
                    DomGeneralElement::Text(x) => {
                        let mut e = next_dom_elem.ok_or(Error::BackendError {
                            msg: "Failed to apply a prerendered node".to_string(),
                            err: None,
                        })?;
                        if state.prev_is_text_node {
                            e = e.next_sibling().ok_or(Error::BackendError {
                                msg: "Failed to apply a prerendered node".to_string(),
                                err: None,
                            })?;
                        } else {
                            state.prev_is_text_node = true;
                        };
                        let ret = e.next_sibling();
                        x.rematch_dom(e);
                        return Ok(ret);
                    }
                    DomGeneralElement::Virtual(_) => ChildMatchKind::Virtual(next_dom_elem),
                    DomGeneralElement::Element(x) => {
                        let e = next_dom_elem.ok_or(Error::BackendError {
                            msg: "Failed to apply a prerendered node".to_string(),
                            err: None,
                        })?;
                        x.rematch_dom(e.clone());
                        state.prev_is_text_node = false;
                        ChildMatchKind::NonVirtual(e)
                    }
                }
            };
            let mut rematch_child = |mut child_dom_elem| -> Result<_, Error> {
                if let Some(mut child) = n.first_child_rc() {
                    loop {
                        let c = {
                            let child_mut = &mut n.borrow_mut(&child);
                            child_dom_elem = rematch_dom(child_mut, child_dom_elem, state)?;
                            child_mut.next_sibling_rc()
                        };
                        match c {
                            None => break,
                            Some(c) => {
                                child = c;
                            }
                        }
                    }
                }
                Ok(child_dom_elem)
            };
            match child_match_kind {
                ChildMatchKind::Virtual(x) => rematch_child(x),
                ChildMatchKind::NonVirtual(x) => {
                    rematch_child(x.first_child())?;
                    state.prev_is_text_node = false;
                    Ok(x.next_sibling())
                }
            }
        }
        let mut tree = self.tree.try_borrow_mut().ok_or(Error::BackendError {
            msg: "Cannot apply prerendered tree while visiting".to_string(),
            err: None,
        })?;
        rematch_dom(
            &mut tree,
            Some(dom_elem.unchecked_into()),
            &mut Default::default(),
        )?;
        Ok(())
    }
}

impl Backend for DomBackend {
    type GeneralElement = DomGeneralElement;
    type VirtualElement = DomVirtualElement;
    type TextNode = DomTextNode;

    fn async_task(fut: impl 'static + std::future::Future<Output = ()>) {
        async_task(fut)
    }

    fn backend_stage(&self) -> BackendStage {
        self.backend_stage
    }

    fn root(&self) -> ForestNode<Self::GeneralElement> {
        self.tree.borrow()
    }

    fn root_mut(&mut self) -> ForestNodeMut<Self::GeneralElement> {
        self.tree.borrow_mut()
    }
}

#[doc(hidden)]
pub enum DomGeneralElement {
    Virtual(DomVirtualElement),
    Text(DomTextNode),
    Element(DomElement),
}

impl std::fmt::Debug for DomGeneralElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Virtual(_) => write!(f, "[Virtual {:p}]", self),
            Self::Text(x) => write!(f, "{:?}", x.text_content(),),
            Self::Element(x) => write!(f, "{:?}", x),
        }
    }
}

impl DomGeneralElement {
    fn is_prerendering(&self) -> dom_state_ty!((), (), ()) {
        match self {
            Self::Virtual(x) => x.is_prerendering(),
            Self::Text(x) => x.is_prerendering(),
            Self::Element(x) => x.is_prerendering(),
        }
    }

    fn create_dom_element<'b>(
        this: &'b mut ForestNodeMut<Self>,
        elem: &'b dom_state_ty!(web_sys::Element, PrerenderingElement, RematchedDomElem),
    ) -> ForestNodeRc<Self> {
        let ret = this.new_tree(Self::Element(unsafe { DomElement::new(elem.clone()) }));
        let token = ret.token();
        if let Self::Element(x) = &mut *this.borrow_mut(&ret) {
            x.init(token);
        } else {
            unreachable!()
        }
        ret
    }

    pub(crate) fn as_dom_element_mut<'b>(
        this: &'b mut ForestNodeMut<Self>,
    ) -> Option<ForestValueMut<'b, DomElement>> {
        if let Self::Element(_) = &mut **this {
            Some(this.map(|g| {
                if let Self::Element(e) = g {
                    e
                } else {
                    unreachable!()
                }
            }))
        } else {
            None
        }
    }

    pub(crate) fn write_inner_html(
        this: &ForestNode<Self>,
        w: &mut impl std::io::Write,
        state: &mut WriteHtmlState,
    ) -> std::io::Result<()> {
        match &**this {
            Self::Text(x) => {
                x.write_inner_html(w, state)?;
            }
            Self::Element(x) => {
                x.write_inner_html(this, w, state)?;
            }
            Self::Virtual(_) => {
                let mut cur = this.first_child();
                while let Some(c) = &cur {
                    Self::write_outer_html(&c, w, state)?;
                    cur = c.next_sibling();
                }
            }
        }
        Ok(())
    }

    pub(crate) fn write_outer_html(
        this: &ForestNode<Self>,
        w: &mut impl std::io::Write,
        state: &mut WriteHtmlState,
    ) -> std::io::Result<()> {
        match &**this {
            Self::Text(x) => {
                x.write_inner_html(w, state)?;
            }
            Self::Element(x) => {
                x.write_outer_html(this, w, state)?;
            }
            Self::Virtual(_) => {
                let mut cur = this.first_child();
                while let Some(c) = &cur {
                    Self::write_outer_html(&c, w, state)?;
                    cur = c.next_sibling();
                }
            }
        }
        Ok(())
    }

    /// Get the inner HTML of the specified node
    #[inline]
    pub fn inner_html(this: &ForestNode<Self>) -> String {
        let mut ret = Vec::new();
        let mut state = Default::default();
        Self::write_inner_html(this, &mut ret, &mut state).unwrap();
        // since all str sources are valid UTF-8, this operation is safe
        unsafe { String::from_utf8_unchecked(ret) }
    }

    /// Get the outer HTML of the specified node
    #[inline]
    pub fn outer_html(this: &ForestNode<Self>) -> String {
        let mut ret = Vec::new();
        let mut state = Default::default();
        Self::write_outer_html(this, &mut ret, &mut state).unwrap();
        // since all str sources are valid UTF-8, this operation is safe
        unsafe { String::from_utf8_unchecked(ret) }
    }
}

impl BackendGeneralElement for DomGeneralElement {
    type BaseBackend = DomBackend;

    #[inline]
    fn as_virtual_element_mut<'b>(
        this: &'b mut ForestNodeMut<Self>,
    ) -> Option<
        ForestValueMut<
            'b,
            <<Self as BackendGeneralElement>::BaseBackend as Backend>::VirtualElement,
        >,
    >
    where
        Self: Sized,
    {
        if let Self::Virtual(_) = &mut **this {
            Some(this.map(|g| {
                if let Self::Virtual(e) = g {
                    e
                } else {
                    unreachable!()
                }
            }))
        } else {
            None
        }
    }

    #[inline]
    fn as_text_node_mut<'b>(
        this: &'b mut ForestNodeMut<Self>,
    ) -> Option<
        ForestValueMut<'b, <<Self as BackendGeneralElement>::BaseBackend as Backend>::TextNode>,
    >
    where
        Self: Sized,
    {
        if let Self::Text(_) = &mut **this {
            Some(this.map(|g| {
                if let DomGeneralElement::Text(e) = g {
                    e
                } else {
                    unreachable!()
                }
            }))
        } else {
            None
        }
    }

    #[inline]
    fn create_virtual_element<'b>(
        this: &'b mut ForestNodeMut<Self>,
    ) -> Result<ForestNodeRc<<Self::BaseBackend as Backend>::GeneralElement>, Error>
    where
        Self: Sized,
    {
        let elem = DomVirtualElement::new(this);
        let child = this.new_tree(Self::Virtual(elem));
        Ok(child)
    }

    #[inline]
    fn create_text_node(
        this: &mut ForestNodeMut<Self>,
        content: &str,
    ) -> Result<ForestNodeRc<<Self::BaseBackend as Backend>::GeneralElement>, Error>
    where
        Self: Sized,
    {
        let elem = DomTextNode::new(this, content);
        let child = this.new_tree(Self::Text(elem));
        Ok(child)
    }

    #[inline]
    fn append<'b>(
        this: &'b mut ForestNodeMut<Self>,
        child: &'b ForestNodeRc<
            <<Self as BackendGeneralElement>::BaseBackend as Backend>::GeneralElement,
        >,
    ) where
        Self: Sized,
    {
        this.append(&child);
        if this.is_prerendering() == DomState::Normal(()) {
            let this = this.as_ref();
            if let Some(parent) = composing::find_nearest_dom_ancestor(this.clone()) {
                let child = this.last_child_rc().unwrap();
                let child = this.borrow(&child);
                let before = composing::find_next_dom_sibling(child.clone());
                let child_frag = composing::collect_child_frag(child);
                if let Some(child_frag) = child_frag.dom() {
                    parent.insert_before(child_frag, before.as_ref()).unwrap();
                }
            }
        }
    }

    #[inline]
    fn insert<'b>(
        this: &'b mut ForestNodeMut<Self>,
        target: &'b ForestNodeRc<
            <<Self as BackendGeneralElement>::BaseBackend as Backend>::GeneralElement,
        >,
    ) where
        Self: Sized,
    {
        this.insert(&target);
        if this.is_prerendering() == DomState::Normal(()) {
            let target = this.as_ref().borrow(&target);
            if let Some(parent) = composing::find_nearest_dom_ancestor(target.clone()) {
                let before = composing::find_next_dom_sibling(target.clone());
                let child_frag = composing::collect_child_frag(target);
                if let Some(child_frag) = child_frag.dom() {
                    parent.insert_before(child_frag, before.as_ref()).unwrap();
                }
            }
        }
    }

    #[inline]
    fn temp_detach(
        this: ForestNodeMut<Self>,
    ) -> ForestNodeRc<<<Self as BackendGeneralElement>::BaseBackend as Backend>::GeneralElement>
    where
        Self: Sized,
    {
        this.detach()
    }

    #[inline]
    fn detach(
        this: ForestNodeMut<Self>,
    ) -> ForestNodeRc<<<Self as BackendGeneralElement>::BaseBackend as Backend>::GeneralElement>
    where
        Self: Sized,
    {
        if this.is_prerendering() == DomState::Normal(()) {
            let this = this.as_ref();
            if let Some(parent) = composing::find_nearest_dom_ancestor(this.clone()) {
                composing::remove_all_children(&parent, this);
            }
        }
        let ret = this.detach();
        ret
    }
}
