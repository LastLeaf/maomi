use crate::{backend::{SupportBackend, Backend}, error::Error};

pub trait Template {
    type Tree;

    fn update(&mut self);
}

pub trait Component<B: Backend> {
    fn create(&mut self) -> B::Component;
    fn update(&mut self, backend_element: B::Component);
}

impl<B: Backend, T: Component<B>> SupportBackend<B> for T {
    fn create(&mut self) -> Result<B::GeneralElement, Error> {
        Ok(self.create().into())
    }

    fn update(&mut self, backend_element: B::GeneralElement) -> Result<(), Error> {
        Ok(self.update(backend_element.try_into().map_err(|_| Error::TreeNotMatchedError)?))
    }
}
