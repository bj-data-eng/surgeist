//! Select decisions: option rules, selection cursors and selectedcontent updates, without a tree or mutation owner.

pub use crate::internal::elements::select::{
    HtmlOptionDisabledAncestor, HtmlOptionSelectAncestorAction, HtmlOptionSelectAncestors,
    HtmlSelectAction, HtmlSelectButton, HtmlSelectChildButton, HtmlSelectCursor,
    HtmlSelectCursorError, HtmlSelectFirstSelectedAction, HtmlSelectFirstSelectedCursor,
    HtmlSelectFirstSelection, HtmlSelectOperation, HtmlSelectOptionRule, HtmlSelectReply,
    HtmlSelectSetterAction, HtmlSelectSetterCursor, HtmlSelectSetterReply,
    HtmlSelectedcontentAncestorAction, HtmlSelectedcontentAncestors, HtmlSelectedcontentUpdate,
    option_disabled_ancestor, select_display_size_default, select_option_rule, select_type_value,
    selectedcontent_update,
};

pub use crate::internal::elements::select_notification::{
    HtmlSelectNotificationAction, HtmlSelectNotificationCursor, HtmlSelectNotificationEvent,
    HtmlSelectNotificationReply, HtmlSelectNotificationReplyError, HtmlSelectUserToggleState,
    HtmlSelectUserToggleTarget, toggle_select_option_for_user,
};
