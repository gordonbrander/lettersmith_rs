use std::env;
use std::path::PathBuf;

pub fn parse_args_to_paths() -> impl Iterator<Item = PathBuf> {
    env::args()
        // Skip the program name (first argument)
        .skip(1)
        .map(PathBuf::from)
}
