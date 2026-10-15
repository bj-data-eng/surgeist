#![forbid(unsafe_code)]

mod contract;

fn main() {
    let authored: surgeist::template::TemplateDocument =
        include!(env!("SURGEIST_TEMPLATE_GENERATED_INPUT"));
    contract::verify(&authored);
    println!("symbolic authored AST, literal spans and revalidation passed");
}
