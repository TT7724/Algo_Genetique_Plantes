//! Modèle d'une plante (schéma v1 validé).

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Plante {
    pub id: String,
    pub nom: String,
    pub famille: String,
    pub hauteur_cm: u32,
    pub espacement_min_cm: u32,
    pub profondeur_racines_cm: u32,
    pub taux_croissance: TauxCroissance,
    pub enracinement_eau: bool,
    pub saisons: Vec<Saison>,
    pub exposition: Exposition,
    pub substrats_compatibles: Vec<String>,
    pub temperature_c: TemperatureC,
    pub besoins_nutriments: BesoinsNutriments,
    pub besoin_eau: Niveau,
    pub rendement: Rendement,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TauxCroissance { Lente, Moyenne, Invasive }

#[derive(Debug, Clone, Copy, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Saison { Printemps, Ete, Automne, Hiver }

#[derive(Debug, Clone, Copy, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Exposition { PleinSoleil, MiOmbre, Ombre }

#[derive(Debug, Clone, Copy, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Niveau { Faible, Moyen, Eleve }

#[derive(Debug, Clone, Deserialize)]
pub struct TemperatureC {
    pub min_survie: i32,
    pub optimal_min: i32,
    pub optimal_max: i32,
    pub max_survie: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BesoinsNutriments {
    pub n: Niveau,
    pub p: Niveau,
    pub k: Niveau,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Rendement {
    pub valeur: f32,
    pub unite: String,
    pub duree_jours: u32,
    pub source: Option<String>,
    pub niveau_confiance: Option<String>,
}
