use surgeist_js::{Source, job::NativeAsyncJob};

fn main() {
    let _job = NativeAsyncJob::new(async |host| {
        let escaped =
            host.with(|mut access| Ok(async move { access.eval(Source::from_bytes("42")) }))?;
        escaped.await
    });
}
