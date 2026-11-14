use super::*;
use crate::document_scripting::guest::tests::attach_public_nodes;

// HTML877d obsolete-elements HTMLFontElement IDL independently specifies this
// declaration order, reflection, and color's LegacyNullToEmptyString conversion.
pub(super) fn font_members_reflect_canonical_attributes() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            (() => {
                const font = document.createElement('font');
                if (!(font instanceof HTMLFontElement)) return 'font interface';
                const members = ['color', 'face', 'size'];
                if (Object.keys(HTMLFontElement.prototype).join() !== members.join())
                    return 'font missing reflected members';
                for (const member of members) {
                    const descriptor = Object.getOwnPropertyDescriptor(HTMLFontElement.prototype, member);
                    if (typeof descriptor?.get !== 'function' || typeof descriptor?.set !== 'function' ||
                        !descriptor.enumerable || !descriptor.configurable ||
                        descriptor.get.length !== 0 || descriptor.set.length !== 1)
                        return 'font descriptor ' + member;
                    if (font[member] !== '') return 'font absent default ' + member;
                    font[member] = null;
                    const expected = member === 'color' ? '' : 'null';
                    if (font[member] !== expected || font.getAttributeNS(null, member) !== expected)
                        return 'font null reflection ' + member;
                }
                return 'ok';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        Ok(())
    }).unwrap();
}
