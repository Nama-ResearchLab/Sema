//! # Sema — "say" in Swahili
//!
//! Offline semantic anchoring for low-resource languages.
//!
//! Your model barely speaks Swahili. Your users only speak Swahili.
//! Sema bridges the gap without a cloud call or a translation model:
//!
//! 1. **Anchor**: resolve surface words to lemmas + English glosses using a
//!    distilled Wiktionary lexicon and language-specific affix rules.
//! 2. (You) reason in English — the strong side of any small model.
//! 3. (You) render output from bilingual templates — never back-translate prose.
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
pub mod lexicon;

pub use lexicon::{LexEntry, Lexicon, Skeleton};

/// Crate version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
