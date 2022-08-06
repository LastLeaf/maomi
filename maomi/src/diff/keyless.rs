use std::marker::PhantomData;

use crate::backend::BackendGeneralElement;
use super::*;

pub struct KeylessList<B: Backend, C> {
    list: Vec<(C, ForestToken)>,
    _phantom: PhantomData<B>,
}

impl<B: Backend, C> KeylessList<B, C> {
    pub fn list_diff_new<'a, 'b>(
        backend_element: &'a mut ForestNodeMut<'b, B::GeneralElement>,
        size_hint: usize,
    ) -> ListKeylessAlgoNew<'a, 'b, B, C> {
        ListKeylessAlgoNew {
            list: Vec::with_capacity(size_hint),
            backend_element,
            _phantom: PhantomData,
        }
    }

    pub fn list_diff_update<'a, 'b>(
        &'a mut self,
        backend_element: &'a mut ForestNodeMut<'b, B::GeneralElement>,
        size_hint: usize,
    ) -> ListKeylessAlgoUpdate<'a, 'b, B, C> {
        if size_hint > self.list.len() {
            self.list.reserve_exact(size_hint - self.list.len());
        }
        ListKeylessAlgoUpdate {
            cur_index: 0,
            list: &mut self.list,
            backend_element,
            _phantom: PhantomData,
        }
    }
}

pub struct ListKeylessAlgoNew<
    'a,
    'b,
    B: Backend,
    C,
> {
    list: Vec<(C, ForestToken)>,
    backend_element: &'a mut ForestNodeMut<'b, B::GeneralElement>,
    _phantom: PhantomData<B>,
}

impl<
    'a,
    'b,
    B: Backend,
    C,
> ListKeylessAlgoNew<'a, 'b, B, C> {
    pub fn next(
        &mut self,
        create_fn: impl FnOnce(&mut ForestNodeMut<B::GeneralElement>) -> Result<C, Error>,
    ) -> Result<(), Error> {
        let backend_element = <B::GeneralElement as BackendGeneralElement>::create_virtual_element(self.backend_element)?;
        let c = create_fn(
            &mut self.backend_element.borrow_mut(&backend_element),
        )?;
        self.list.push((c, backend_element.token()));
        <B::GeneralElement as BackendGeneralElement>::append(
            self.backend_element,
            &backend_element,
        );
        Ok(())
    }

    pub fn end(self) -> KeylessList<B, C> {
        KeylessList {
            list: self.list,
            _phantom: PhantomData,
        }
    }
}

pub struct ListKeylessAlgoUpdate<
    'a,
    'b,
    B: Backend,
    C,
> {
    cur_index: usize,
    list: &'a mut Vec<(C, ForestToken)>,
    backend_element: &'a mut ForestNodeMut<'b, B::GeneralElement>,
    _phantom: PhantomData<B>,
}

impl<
    'a,
    'b,
    B: Backend,
    C,
> ListKeylessAlgoUpdate<'a, 'b, B, C> {
    pub fn next(
        &mut self,
        create_fn: impl FnOnce(&mut ForestNodeMut<B::GeneralElement>) -> Result<C, Error>,
        update_fn: impl FnOnce(&mut C, &mut ForestNodeMut<B::GeneralElement>) -> Result<(), Error>,
    ) -> Result<(), Error> {
        if let Some((ref mut c, forest_token)) = self.list.get_mut(self.cur_index) {
            update_fn(
                c,
                &mut self.backend_element.borrow_mut_token(&forest_token),
            )?;
        } else {
            let backend_element = <B::GeneralElement as BackendGeneralElement>::create_virtual_element(self.backend_element)?;
            let c = create_fn(
                &mut self.backend_element.borrow_mut(&backend_element),
            )?;
            self.list.push((c, backend_element.token()));
            <B::GeneralElement as BackendGeneralElement>::append(
                self.backend_element,
                &backend_element,
            );
        }
        self.cur_index += 1;
        Ok(())
    }

    pub fn end(self) -> Result<(), Error> {
        for (_c, forest_token) in self.list.drain(self.cur_index..) {
            let _ = <B::GeneralElement as BackendGeneralElement>::detach(
                self.backend_element.borrow_mut_token(&forest_token),
            );
        }
        Ok(())
    }
}

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

impl<'a, B: Backend, C> IntoIterator for &'a KeylessList<B, C> {
    type Item = &'a C;
    type IntoIter = KeylessListIter<'a, C>;

    fn into_iter(self) -> Self::IntoIter {
        KeylessListIter {
            children: self.list.iter(),
        }
    }
}
