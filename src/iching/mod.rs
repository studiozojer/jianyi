pub mod cast;
pub mod compose;
pub mod data;
pub mod hexagram_data;
pub mod render;
pub mod trigram;

#[cfg(feature = "cli")]
pub mod cli;
#[cfg(feature = "cli")]
pub mod cmd_cast;
#[cfg(feature = "cli")]
pub mod cmd_compose;
#[cfg(feature = "cli")]
pub mod cmd_hexagram;
#[cfg(feature = "cli")]
pub mod cmd_trigram;

pub use cast::{cast, CastResult, LineValue};
pub use compose::{parse as parse_expr, Expr};
pub use data::{lookup_hexagram, HexagramRecord, TrigramRecord};
pub use hexagram_data::moving_lines;
pub use render::{lines_from_binary, render_lines};
pub use trigram::{hexagrams_with_lower, hexagrams_with_upper, lookup_trigram};
