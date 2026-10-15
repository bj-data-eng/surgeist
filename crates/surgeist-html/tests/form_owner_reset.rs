//! Source-ordered observations of HTML877d #reset-the-form-owner.
//! The cursor has no tree; real candidate lookup and peer-state regressions
//! additionally execute through the consuming parser and semantics owners.

use surgeist_html::bounded::{
    HtmlFormOwnerLookup as Lookup, HtmlFormOwnerResetAction as A,
    HtmlFormOwnerResetCursor as Cursor, HtmlFormOwnerResetCursorError as Error,
    HtmlFormOwnerResetReply as R, form_owner_lookup,
};

fn trace(cursor: &mut Cursor, steps: &[(A, R)]) {
    for &(action, reply) in steps {
        assert_eq!(cursor.action(), action);
        cursor.reply(reply).unwrap();
    }
    assert_eq!(cursor.action(), A::Complete);
}

#[test]
fn first_id_selection_requires_both_explicit_listed_form_and_connectedness() {
    assert_eq!(form_owner_lookup(true, true), Lookup::FirstId);
    assert_eq!(form_owner_lookup(true, false), Lookup::NearestAncestor);
    for ignored_connectedness in [false, true] {
        assert_eq!(
            form_owner_lookup(false, ignored_connectedness),
            Lookup::NearestAncestor
        );
    }
}

#[test]
fn parser_flag_clears_before_lookup_and_nonlisted_nearest_owner_is_retained() {
    trace(
        &mut Cursor::new(false, true),
        &[
            (A::ClearParserInserted, R::Applied),
            (
                A::ReadCurrentOwnerIsNearestAncestor,
                R::CurrentOwnerIsNearestAncestor(true),
            ),
        ],
    );
}

#[test]
fn listed_owner_retention_checks_attribute_presence_before_nearest_ancestor() {
    trace(
        &mut Cursor::new(true, true),
        &[
            (A::ClearParserInserted, R::Applied),
            (A::ReadFormAttributePresence, R::FormAttributePresent(false)),
            (
                A::ReadCurrentOwnerIsNearestAncestor,
                R::CurrentOwnerIsNearestAncestor(true),
            ),
        ],
    );
}

#[test]
fn absent_owner_skips_retention_and_old_owner_effects_then_uses_fresh_ancestor() {
    trace(
        &mut Cursor::new(false, false),
        &[
            (A::ClearParserInserted, R::Applied),
            (A::ClearOwner, R::Applied),
            (A::FindNearestAncestor, R::FormFound(true)),
            (A::AssociateFoundOwner, R::Applied),
            (A::ApplyOwnerChangeEffects, R::Applied),
        ],
    );
}

#[test]
fn failed_retention_clears_owner_and_effects_before_fresh_ancestor_lookup() {
    trace(
        &mut Cursor::new(false, true),
        &[
            (A::ClearParserInserted, R::Applied),
            (
                A::ReadCurrentOwnerIsNearestAncestor,
                R::CurrentOwnerIsNearestAncestor(false),
            ),
            (A::ClearOwner, R::Applied),
            (A::ApplyOwnerChangeEffects, R::Applied),
            (A::FindNearestAncestor, R::FormFound(false)),
        ],
    );
}

#[test]
fn connected_explicit_nonform_first_id_ends_null_without_ancestor_fallback() {
    trace(
        &mut Cursor::new(true, true),
        &[
            (A::ClearParserInserted, R::Applied),
            (A::ReadFormAttributePresence, R::FormAttributePresent(true)),
            (A::ClearOwner, R::Applied),
            (A::ApplyOwnerChangeEffects, R::Applied),
            (A::ReadConnectedness, R::Connected(true)),
            (A::FindFirstIdMatch, R::FormFound(false)),
        ],
    );
}

#[test]
fn disconnected_explicit_form_uses_nearest_ancestor_instead_of_id_lookup() {
    trace(
        &mut Cursor::new(true, false),
        &[
            (A::ClearParserInserted, R::Applied),
            (A::ReadFormAttributePresence, R::FormAttributePresent(true)),
            (A::ClearOwner, R::Applied),
            (A::ReadConnectedness, R::Connected(false)),
            (A::FindNearestAncestor, R::FormFound(true)),
            (A::AssociateFoundOwner, R::Applied),
            (A::ApplyOwnerChangeEffects, R::Applied),
        ],
    );
}

#[test]
fn same_final_owner_still_applies_effects_at_null_then_associated_state() {
    let mut cursor = Cursor::new(true, true);
    let mut parser_inserted = true;
    let mut owner = Some(17);
    let mut observed_effect_owners = Vec::new();
    // The literal source trace requires two real owner changes even when the
    // final candidate equals the original. It must not collapse to a no-op.
    for (action, reply) in [
        (A::ClearParserInserted, R::Applied),
        (A::ReadFormAttributePresence, R::FormAttributePresent(true)),
        (A::ClearOwner, R::Applied),
        (A::ApplyOwnerChangeEffects, R::Applied),
        (A::ReadConnectedness, R::Connected(true)),
        (A::FindFirstIdMatch, R::FormFound(true)),
        (A::AssociateFoundOwner, R::Applied),
        (A::ApplyOwnerChangeEffects, R::Applied),
    ] {
        assert_eq!(cursor.action(), action);
        match action {
            A::ClearParserInserted => parser_inserted = false,
            A::ClearOwner => owner = None,
            A::AssociateFoundOwner => owner = Some(17),
            A::ApplyOwnerChangeEffects => observed_effect_owners.push(owner),
            _ => assert!(!parser_inserted),
        }
        cursor.reply(reply).unwrap();
    }
    assert_eq!(cursor.action(), A::Complete);
    assert!(!parser_inserted);
    assert_eq!(owner, Some(17));
    assert_eq!(observed_effect_owners, [None, Some(17)]);
}

#[test]
fn wrong_replies_preserve_pending_state_and_its_valid_continuation() {
    let mut cursor = Cursor::new(true, true);
    for (action, right_reply) in [
        (A::ClearParserInserted, R::Applied),
        (A::ReadFormAttributePresence, R::FormAttributePresent(false)),
        (
            A::ReadCurrentOwnerIsNearestAncestor,
            R::CurrentOwnerIsNearestAncestor(false),
        ),
        (A::ClearOwner, R::Applied),
        (A::ApplyOwnerChangeEffects, R::Applied),
        (A::FindNearestAncestor, R::FormFound(true)),
        (A::AssociateFoundOwner, R::Applied),
        (A::ApplyOwnerChangeEffects, R::Applied),
    ] {
        assert_eq!(cursor.action(), action);
        let wrong = if matches!(right_reply, R::Applied) {
            R::Connected(false)
        } else {
            R::Applied
        };
        assert_eq!(cursor.reply(wrong), Err(Error::Reply));
        assert_eq!(cursor.action(), action);
        cursor.reply(right_reply).unwrap();
    }
    assert_eq!(cursor.action(), A::Complete);
    assert_eq!(cursor.reply(R::Applied), Err(Error::Reply));
    assert_eq!(cursor.action(), A::Complete);
}
