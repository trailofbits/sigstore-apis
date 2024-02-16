use progenitor::{GenerationSettings, Generator, TypeImpl};

fn generate_for_service(service: &str) {
    let src = format!("openapi/{service}.openapi.json");
    println!("cargo:rerun-if-changed={}", src);
    let file = std::fs::File::open(src).unwrap();
    let spec = serde_json::from_reader(file).unwrap();

    let mut generator = Generator::new(GenerationSettings::default().with_replacement(
        "ProposedEntry",
        "crate::types::ProposedEntry",
        [TypeImpl::Display].into_iter(),
    ));

    let tokens = generator.generate_tokens(&spec).unwrap();
    let ast = syn::parse2(tokens).unwrap();
    let content = prettyplease::unparse(&ast);

    let mut out_file = std::path::Path::new(&std::env::var("OUT_DIR").unwrap()).to_path_buf();
    out_file.push(format!("codegen_{service}.rs"));

    std::fs::write(out_file, content).unwrap();
}

fn main() {
    for service in ["fulcio", "rekor"] {
        generate_for_service(service);
    }
}
