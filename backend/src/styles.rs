use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct StyleInfo {
    pub name: String,
    pub description: String,
    pub sample_seed: String,
}

pub fn available_styles() -> Vec<StyleInfo> {
    vec![
        StyleInfo {
            name: "geometric".to_string(),
            description: "5×5 grid with vertical symmetry. Clean identicon style.".to_string(),
            sample_seed: "nanook".to_string(),
        },
        StyleInfo {
            name: "rings".to_string(),
            description: "Concentric colored rings. Abstract and distinctive.".to_string(),
            sample_seed: "nanook".to_string(),
        },
        StyleInfo {
            name: "robot".to_string(),
            description: "Procedural robot faces. Agent-themed with varying features.".to_string(),
            sample_seed: "nanook".to_string(),
        },
        StyleInfo {
            name: "blockies".to_string(),
            description: "8×8 colored grid. Ethereum-style block avatars.".to_string(),
            sample_seed: "nanook".to_string(),
        },
        StyleInfo {
            name: "gradient".to_string(),
            description: "Two-color gradient with geometric overlay shape.".to_string(),
            sample_seed: "nanook".to_string(),
        },
    ]
}

pub fn is_valid_style(style: &str) -> bool {
    matches!(style, "geometric" | "rings" | "robot" | "blockies" | "gradient")
}

pub const DEFAULT_STYLE: &str = "geometric";
pub const DEFAULT_SIZE: u32 = 256;
pub const MIN_SIZE: u32 = 16;
pub const MAX_SIZE: u32 = 1024;
