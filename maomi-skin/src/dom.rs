use crate::StyleSheetConstructor;
use crate::parser::CssToken;

pub(crate) struct DomStyleSheet {}

impl StyleSheetConstructor for DomStyleSheet {
    type PropertyValue = Vec<CssToken>;
    type FontFacePropertyValue = Vec<CssToken>;

    fn construct_sheet() {
        todo!()
    }
}
