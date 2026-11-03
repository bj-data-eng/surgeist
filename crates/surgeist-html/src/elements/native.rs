//! Native initialization, custom element state, element reflection and design mode, shared by several element families.

pub use crate::internal::elements::custom_element_state::{
    HtmlCustomElementState, custom_element_state_without_definition,
};
pub use crate::internal::elements::design_mode::{
    HtmlDesignModeAction, HtmlDesignModeCursor, HtmlDesignModeError, HtmlDesignModeInput,
    HtmlDesignModeReply,
};
pub use crate::internal::elements::native_initialization::{
    HtmlClosedElementState, HtmlCommonHtmlState, HtmlCommonScope, HtmlCommonState,
    HtmlElementClass, HtmlNativeFreshNonce, HtmlNativeFreshSlot, HtmlNativeInitialization,
};
pub use crate::internal::elements::reflected_element::{
    HtmlReflectedElementSetter, HtmlReflectedElementSetterEffect,
    reflected_element_attribute_change,
};
