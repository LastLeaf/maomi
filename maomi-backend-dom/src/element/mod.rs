use maomi::backend::SupportBackend;

use crate::DomBackend;

#[allow(non_camel_case_types)]
pub struct div {}

impl div {
    pub fn set_property_hidden() {
        todo!()
    }
}

impl SupportBackend<DomBackend> for div {
    fn create(&mut self) -> Result<<DomBackend as maomi::backend::Backend>::GeneralElement, maomi::error::Error> {
        todo!()
    }

    fn update(&mut self, backend_element: <DomBackend as maomi::backend::Backend>::GeneralElement) -> Result<(), maomi::error::Error> {
        todo!()
    }
}
