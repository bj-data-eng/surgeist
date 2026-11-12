use super::*;

pub(in crate::document_scripting) fn window_factory_creates_the_exact_default_policy() {
    let mut host = public_node_host(5_000, 20_000);
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        assert_guest(scope, r#"
            (() => {
                const factory = globalThis.trustedTypes;
                if (typeof factory !== 'object' || factory === null || factory !== window.trustedTypes ||
                    factory.defaultPolicy !== null) return false;
                const visited = [];
                const callback = input => input;
                const options = {
                    get createHTML() { visited.push('html'); return callback; },
                    get createParserOptions() { visited.push('parser'); },
                    get createScript() { visited.push('script'); return callback; },
                    get createScriptURL() { visited.push('url'); return callback; }
                };
                const unusual = factory.createPolicy('A\ud800\0', options);
                if (unusual.name !== 'A\ud800\0' || factory.defaultPolicy !== null ||
                    visited.join(',') !== 'html,parser,script,url') return false;
                const original = {};
                try {
                    factory.createPolicy('default', {get createHTML() { throw original; }});
                    return false;
                } catch (error) { if (error !== original) return false; }
                const policy = factory.createPolicy('default', options);
                if (policy.name !== 'default' || policy !== factory.defaultPolicy) return false;
                try { factory.createPolicy('default'); return false; }
                catch (error) { return error instanceof TypeError && factory.defaultPolicy === policy; }
            })()
        "#)
    })
    .unwrap();
}
