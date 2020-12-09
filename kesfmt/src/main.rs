use rayon::prelude::*;
use std::io::Write;

fn main() {
    glob::glob("**/*.kes")
        .expect("glob files")
        .par_bridge()
        .filter_map(Result::ok)
        .try_for_each(|path| -> Result<(), kes::formatter::FormatError> {
            let source = std::fs::read_to_string(&path)?;

            let mut out = std::fs::File::create(&path)?;

            kes::formatter::format_code(&source, &out)?;

            out.flush()?;

            Ok(())
        })
        .unwrap();
}
