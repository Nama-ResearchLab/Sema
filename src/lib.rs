//! # Sema -- "say" in Swahili
//!
//! Offline semantic anchoring for low-resource languages.
//!
//! Your model barely speaks Swahili. Your users only speak Swahili.
//! Sema bridges the gap without a cloud call or a translation model:
//!
//! 1. **Anchor**: resolve surface words to lemmas + English glosses using a
//!    distilled Wiktionary lexicon and language-specific affix rules.
//! 2. **Linearize**: transpose resolved morphology into telegraphic English
//!    so the model processes clean, low-token semantic input.
//! 3. **Race**: cascade multiple lightweight parsers; fastest good-enough
//!    result wins (early-exit waterfall).
//! 4. **Reason**: the LLM works in English -- the strong side of any small model.
//! 5. **Rolodex**: compile the model's English intent back into grammatical
//!    Swahili via deterministic slot assembly.
//! 6. **Render**: bilingual output via gloss reverse-index -- never
//!    back-translate prose.
//!
//! ```no_run
//! use sema::Lexicon;
//!
//! let lex = Lexicon::load("data/swahili.distilled.jsonl")?;
//! let sk = lex.skeleton_for("umefikia").unwrap();
//! assert_eq!(sk.gloss.to_lowercase().contains("arrive"), true);
//! # Ok::<(), std::io::Error>(())
//! ```

pub mod affix;
pub mod backpass;
pub mod lexicon;
pub mod linearizer;
pub mod racer;
pub mod render;
pub mod rolodex;

pub use lexicon::{LexEntry, Lexicon, Skeleton};
pub use linearizer::{Linearized, MorphDecomp, Token};
pub use racer::{RaceResult, Racer};
pub use render::{RenderIndex, render_bilingual, render_simple};
pub use rolodex::{Intent, Rolodex, Synthesized};

/// Crate version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
