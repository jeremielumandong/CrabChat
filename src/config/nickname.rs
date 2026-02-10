//! Random nickname generator.
//!
//! Produces nicknames in the format `AdjectiveNounNN` (e.g. `NeonFox42`),
//! which fit within IRC's typical 9-character nickname limit.

use rand::RngExt;

const ADJECTIVES: &[&str] = &[
    "Shadow", "Neon", "Cyber", "Lunar", "Solar", "Frost", "Storm", "Dark", "Pixel", "Ghost",
    "Hyper", "Turbo", "Stealth", "Cosmic", "Iron", "Velvet", "Crimson", "Silent", "Rogue",
    "Mystic", "Atomic", "Rapid", "Zero", "Nova", "Onyx", "Cobalt", "Azure", "Hex", "Glitch",
    "Wired", "Chrome", "Prism",
];

const NOUNS: &[&str] = &[
    "Fox", "Wolf", "Hawk", "Raven", "Lynx", "Viper", "Shark", "Falcon", "Panda", "Tiger", "Cobra",
    "Owl", "Phoenix", "Dragon", "Jaguar", "Mantis", "Sphinx", "Kraken", "Otter", "Hound", "Crow",
    "Bear", "Panther", "Coyote", "Moth", "Newt", "Crane", "Bison", "Dingo", "Reef", "Byte", "Node",
];

/// Generate a random nickname like `NeonFox42` (max 9 chars fits IRC limits).
pub fn generate_nickname() -> String {
    let mut rng = rand::rng();
    let adj = ADJECTIVES[rng.random_range(0..ADJECTIVES.len())];
    let noun = NOUNS[rng.random_range(0..NOUNS.len())];
    let num: u8 = rng.random_range(0..100);
    format!("{}{}{}", adj, noun, num)
}
