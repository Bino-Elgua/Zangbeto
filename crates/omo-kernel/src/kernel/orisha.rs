use serde::{Serialize, Deserialize};
use schemars::JsonSchema;

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum OrishaMask {
    #[serde(rename = "Èṣù")]
    Eshu,
    #[serde(rename = "Ọ̀ṣun")]
    Oshun,
    #[serde(rename = "Yemọja")]
    Yemoja,
    #[serde(rename = "Ọbàtálá")]
    Obatala,
    #[serde(rename = "Ògún")]
    Ogun,
    #[serde(rename = "Ọya")]
    Oya,
    #[serde(rename = "Ṣàngó")]
    Shango,
}

impl std::fmt::Display for OrishaMask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrishaMask::Eshu => write!(f, "Èṣù"),
            OrishaMask::Oshun => write!(f, "Ọ̀ṣun"),
            OrishaMask::Yemoja => write!(f, "Yemọja"),
            OrishaMask::Obatala => write!(f, "Ọbàtálá"),
            OrishaMask::Ogun => write!(f, "Ògún"),
            OrishaMask::Oya => write!(f, "Ọya"),
            OrishaMask::Shango => write!(f, "Ṣàngó"),
        }
    }
}

impl Default for OrishaMask {
    fn default() -> Self {
        OrishaMask::Eshu
    }
}
