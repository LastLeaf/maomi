//! The keyless list algorithm module.
//! 
//! This is one of the list compare algorithm.
//! See [diff](../) module documentation for details.
//! 

use std::marker::PhantomData;

use super::*;
use crate::backend::BackendGeneralElement;

/// The repeated list storing the list state.
/// 
/// It is auto-managed by the `#[component]` .
/// Do not touch unless you know how it works exactly.
pub struct KeylessList<C> {
    list: Vec<(C, ForestToken)>,
}

impl<C> KeylessList<C> {
    #[doc(hidden)]
    #[inline]
    pub fn list_diff_new<'a, 'b, B: Backend>(
        backend_element: &'a mut ForestNodeMut<'b, B::GeneralElement>,
        size_hint: usize,
    ) -> ListAlgo<ListKeylessAlgoNew<'a, 'b, B, C>, ListKeylessAlgoUpdate<'a, 'b, B, C>> {
        ListAlgo::New(
            ListKeylessAlgoNew {
                list: Vec::with_capacity(size_hint),
                backend_element,
                _phantom: PhantomData,
            }
        )
    }

    #[doc(hidden)]
    #[inline]
    pub fn list_diff_update<'a, 'b, B: Backend>(
        &'a mut self,
        backend_element: &'a mut ForestNodeMut<'b, B::GeneralElement>,
        size_hint: usize,
    ) -> ListAlgo<ListKeylessAlgoNew<'a, 'b, B, C>, ListKeylessAlgoUpdate<'a, 'b, B, C>> {
        if size_hint > self.list.len() {
            self.list.reserve_exact(size_hint - self.list.len());
        }
        ListAlgo::Update(
            ListKeylessAlgoUpdate {
                cur_index: 0,
                list: &mut self.list,
                backend_element,
                _phantom: PhantomData,
            }
        )
    }
}

#[doc(hidden)]
pub struct ListKeylessAlgoNew<'a, 'b, B: Backend, C> {
    list: Vec<(C, ForestToken)>,
    backend_element: &'a mut ForestNodeMut<'b, B::GeneralElement>,
    _phantom: PhantomData<B>,
}

impl<'a, 'b, B: Backend, C> ListKeylessAlgoNew<'a, 'b, B, C> {
    #[doc(hidden)]
    pub fn next(
        &mut self,
        create_fn: impl FnOnce(&mut ForestNodeMut<B::GeneralElement>) -> Result<C, Error>,
    ) -> Result<(), Error> {
        let backend_element = <B::GeneralElement as BackendGeneralElement>::create_virtual_element(
            self.backend_element,
        )?;
        let c = create_fn(&mut self.backend_element.borrow_mut(&backend_element))?;
        self.list.push((c, backend_element.token()));
        <B::GeneralElement as BackendGeneralElement>::append(
            self.backend_element,
            &backend_element,
        );
        Ok(())
    }

    #[doc(hidden)]
    #[inline]
    pub fn end(self) -> KeylessList<C> {
        KeylessList {
            list: self.list,
        }
    }
}

#[doc(hidden)]
pub struct ListKeylessAlgoUpdate<'a, 'b, B: Backend, C> {
    cur_index: usize,
    list: &'a mut Vec<(C, ForestToken)>,
    backend_element: &'a mut ForestNodeMut<'b, B::GeneralElement>,
    _phantom: PhantomData<B>,
}

impl<'a, 'b, B: Backend, C> ListKeylessAlgoUpdate<'a, 'b, B, C> {
    #[doc(hidden)]
    pub fn next(
        &mut self,
        create_or_update_fn: impl FnOnce(
            Option<&mut C>,
            &mut ForestNodeMut<B::GeneralElement>,
        ) -> Result<Option<C>, Error>,
    ) -> Result<(), Error> {
        if let Some((ref mut c, forest_token)) = self.list.get_mut(self.cur_index) {
            if let Some(n) = &mut self.backend_element.borrow_mut_token(&forest_token) {
                create_or_update_fn(Some(c), n)?;
            }
        } else {
            let backend_element =
                <B::GeneralElement as BackendGeneralElement>::create_virtual_element(
                    self.backend_element,
                )?;
            let c =
                create_or_update_fn(None, &mut self.backend_element.borrow_mut(&backend_element))?
                    .ok_or(Error::ListChangeWrong)?;
            self.list.push((c, backend_element.token()));
            <B::GeneralElement as BackendGeneralElement>::append(
                self.backend_element,
                &backend_element,
            );
        }
        self.cur_index += 1;
        Ok(())
    }

    #[doc(hidden)]
    pub fn end(self) -> Result<(), Error> {
        for (_c, forest_token) in self.list.drain(self.cur_index..) {
            if let Some(n) = self.backend_element.borrow_mut_token(&forest_token) {
                <B::GeneralElement as BackendGeneralElement>::detach(n);
            }
        }
        Ok(())
    }
}

/// The iterator for a `KeylessList`
pub struct KeylessListIter<'a, C> {
    children: std::slice::Iter<'a, (C, ForestToken)>,
}

impl<'a, C> Iterator for KeylessListIter<'a, C> {
    type Item = &'a C;

    fn next(&mut self) -> Option<Self::Item> {
        self.children.next().map(|(x, _)| x)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.children.size_hint()
    }
}

impl<'a, C> IntoIterator for &'a KeylessList<C> {
    type Item = &'a C;
    type IntoIter = KeylessListIter<'a, C>;

    fn into_iter(self) -> Self::IntoIter {
        KeylessListIter {
            children: self.list.iter(),
        }
    }
}
