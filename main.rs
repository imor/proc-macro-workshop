// Write code here.
//
// To see what the code looks like after macro expansion:
//     $ cargo expand
//
// To run the code:
//     $ cargo run

use derive_debug::CustomDebug;

#[derive(CustomDebug)]
pub struct Field {
    pub name: &'static str,
    pub bitmask: u16,
}

fn main() {}
