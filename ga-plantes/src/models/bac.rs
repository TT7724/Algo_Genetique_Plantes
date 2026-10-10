//! Modèle d'un bac de culture (schéma v1 validé).

use serde::Deserialize;

use super::plant::{Exposition, Saison};

#[derive(Debug, Clone, Deserialize)]
pub struct Bac {
    pub nom: String,
    pub dimensions_cm: DimensionsCm,
    pub hauteur_terre_cm: u32,
    pub marge_bord_cm: u32,
    pub substrats: Vec<String>,
    pub compost_disponible: bool,
    pub drainage_eau: bool,
    pub exposition: Exposition,
    pub saison_cible: Saison,
    pub temp_moyenne_c: TempMoyenneC,
    pub protection: Protection,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DimensionsCm {
    pub longueur: u32,
    pub largeur: u32,
    pub hauteur: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TempMoyenneC {
    pub min: i32,
    pub max: i32,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Protection { Exterieur, SerreFroide, SerreChauffee }

impl Bac {
    /// Surface utile = (L - 2·marge) × (l - 2·marge), en cm².
    pub fn surface_utile_cm2(&self) -> i64 {
        let l = self.dimensions_cm.longueur as i64 - 2 * self.marge_bord_cm as i64;
        let w = self.dimensions_cm.largeur as i64 - 2 * self.marge_bord_cm as i64;
        l * w
    }
}
