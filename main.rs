// Write code here.
//
// To see what the code looks like after macro expansion:
//     $ cargo expand
//
// To run the code:
//     $ cargo run

use derive_builder::Builder;

#[derive(Builder)]
struct Test {
    executable: String,
    args: Vec<String>,
    env: Vec<String>,
    current_dir: String,
    test: std::option::Option<String>,
}

fn main() {}
