use super::*;

pub(in crate::document_scripting) fn window_factory_creates_the_exact_default_policy() {
    let mut host = public_node_host(5_000, 20_000);
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        let result = scope.evaluate(&units(r#"
            (() => {
                const factory = globalThis.trustedTypes;
                if (typeof factory !== 'object' || factory === null || factory !== window.trustedTypes ||
                    factory.defaultPolicy !== null) return 'factory exposure';
                const visited = [];
                const callback = input => input;
                const options = {
                    get createHTML() { visited.push('html'); return callback; },
                    get createParserOptions() { visited.push('parser'); },
                    get createScript() { visited.push('script'); return callback; },
                    get createScriptURL() { visited.push('url'); return callback; }
                };
                const unusual = factory.createPolicy('A\ud800\0', options);
                if (unusual.name !== 'A\ud800\0') return 'name:' + Array.from(unusual.name ?? '').map(c => c.charCodeAt(0));
                if (factory.defaultPolicy !== null) return 'unusual name became default';
                if (visited.join(',') !== 'html,parser,script,url') return 'dictionary order:' + visited.join(',');
                const original = {};
                try {
                    factory.createPolicy('default', {get createHTML() { throw original; }});
                    return 'getter returned';
                } catch (error) { if (error !== original) return 'getter throw identity'; }
                const policy = factory.createPolicy('default', options);
                if (policy.name !== 'default' || policy !== factory.defaultPolicy) return 'default identity';
                try { factory.createPolicy('default'); return 'duplicate accepted'; }
                catch (error) { return error instanceof TypeError && factory.defaultPolicy === policy ? 'ok' : 'duplicate rejection'; }
            })()
        "#))?;
        let status = String::from_utf16_lossy(&scope.to_utf16(&result)?);
        assert_eq!(status, "ok");
        Ok(())
    })
    .unwrap();
}
