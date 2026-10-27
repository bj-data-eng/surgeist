//! Declarations, declaration lists, custom properties, substitution and importance.

pub(crate) mod declaration_serialization;
pub(crate) mod specified_declaration_block;
pub(crate) mod syntax;

#[doc(inline)]
pub use specified_declaration_block::{
    CssDeclarationBlockError, CssInvalidKeyframeSourceReason, CssSpecifiedDeclarationBlock,
    CssSpecifiedDeclarationEntry, CssSpecifiedDeclarationValueRef,
};
#[doc(inline)]
pub use syntax::{
    CssAuthoredDeclarationValue, CssAuthoredFunctionArguments, CssCustomDeclaration,
    CssCustomPropertyDeclaredValue, CssCustomPropertyName, CssCustomPropertyValue, CssDeclaration,
    CssDeclarationBody, CssDeclarationList, CssGlobalKeyword, CssImportance,
    CssSubstitutionDependentValue, CssVariableFallback, CssVariableReference,
};
