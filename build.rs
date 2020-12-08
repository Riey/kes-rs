fn main() {
    // lalrpop::process_root().unwrap();
    lalrpop::Configuration::new()
        .always_use_colors()
        .emit_rerun_directives(true)
        .process_current_dir()
        .unwrap();
}
