use std::path::Path;

use reli::core::validation::validate_phase0_artifacts;

fn main() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    match validate_phase0_artifacts(root) {
        Ok(summary) => {
            println!(
                "phase0 validation succeeded: checked {} artifacts",
                summary.checked
            );
        }
        Err(issue) => {
            eprintln!("phase0 validation failed for {}", issue.target);
            eprintln!("details: {}", issue.details);
            std::process::exit(1);
        }
    }
}
