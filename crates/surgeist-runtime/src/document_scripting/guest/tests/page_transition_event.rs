use super::*;

#[test]
fn page_transition_event_converts_inherited_flags_and_exposes_branded_persisted() {
    let mut host = public_node_host(20_000, 200_000);
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        assert_guest(scope, "typeof PageTransitionEvent === 'function'")?;
        assert_guest(
            scope,
            r#"(() => {
                const keys = [];
                const init = new Proxy({ bubbles: true, cancelable: true, composed: true, persisted: 1 }, {
                    get(target, key) { keys.push(key); return target[key]; }
                });
                const event = new PageTransitionEvent("authored", init);
                const defaulted = new PageTransitionEvent("defaulted");
                const getter = Object.getOwnPropertyDescriptor(PageTransitionEvent.prototype, "persisted").get;
                let wrongBrand = false;
                try { getter.call(new Event("ordinary")); }
                catch (error) { wrongBrand = error instanceof TypeError; }
                return event instanceof Event && event instanceof PageTransitionEvent &&
                    Object.getPrototypeOf(PageTransitionEvent.prototype) === Event.prototype &&
                    event.type === "authored" && event.persisted === true &&
                    event.bubbles === true && event.cancelable === true && event.composed === true &&
                    event.isTrusted === false && defaulted.persisted === false && wrongBrand &&
                    keys.join(",") === "bubbles,cancelable,composed,persisted";
            })()"#,
        )
    })
    .unwrap();
}
