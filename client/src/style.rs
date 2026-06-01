pub fn generate() -> &'static str {
    include_str!(concat!(env!("OUT_DIR"), "/generated.css"))
}
