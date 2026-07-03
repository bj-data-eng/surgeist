use std::{env, fs, process};

use surgeist_fixture_metadata::{
    FixtureMetadataRequest, INLINE_METRICS_SCHEMA_LIMITATION, generate_layout_ready_metadata,
};

fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();
    if !(4..=5).contains(&args.len()) {
        eprintln!(
            "usage: surgeist-fixture-metadata <identity> <source-path> <generated-path> \
             <css-file> [expectation]"
        );
        process::exit(2);
    }

    let css_source = match fs::read_to_string(&args[3]) {
        Ok(source) => source,
        Err(error) => {
            eprintln!("failed to read CSS source {}: {error}", args[3]);
            process::exit(1);
        }
    };

    let request = FixtureMetadataRequest {
        identity: args[0].clone(),
        source_path: args[1].clone(),
        generated_path: args[2].clone(),
        css_source,
        expectation: args.get(4).cloned(),
    };

    let metadata = match generate_layout_ready_metadata(request) {
        Ok(metadata) => metadata,
        Err(error) => {
            eprintln!("failed to generate fixture metadata: {error}");
            process::exit(1);
        }
    };

    println!("schema_version={}", metadata.schema_version().get());
    println!("identity={}", metadata.identity().as_str());
    println!("source={}", metadata.artifacts().source().as_str());
    println!("generated={}", metadata.artifacts().generated().as_str());
    println!("status={:?}", metadata.status().kind());
    if let Some(expectation) = metadata.expectation() {
        println!("expectation={}", expectation.as_str());
    }
    println!("generator={}", metadata.provenance().generator());
    if let Some(adapter) = metadata.provenance().adapter() {
        println!("adapter={adapter}");
    }
    println!("schema_limitation={INLINE_METRICS_SCHEMA_LIMITATION}");
}
