use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

use crate::models::{Bac, Plante};

// --- Structs pour les données ---
#[derive(Debug, Deserialize, Clone)]
pub struct Interaction {
    pub plante_a: String,
    pub plante_b: String,
    pub r#type: InteractionType,
    pub intensite: Intensite,
    pub confiance: Confiance,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum InteractionType {
    Compagnonnage,
    CompetitionRacinaire,
    CompetitionEau,
    CompetitionLumiere,
    Neutre,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Intensite {
    Faible,
    Moyenne,
    Forte,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Confiance {
    Faible,
    Moyenne,
}

/// Configuration AG — miroir exact de `data/config.json`.
#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub algorithme: Algorithme,
    pub chemins: Chemins,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Algorithme {
    pub population: usize,
    pub generations_max: usize,
    pub taux_croisement: f64,
    pub taux_mutation: f64,
    pub elitisme: usize,
    pub selection: String,
    pub seed: u64,
    pub early_stopping: EarlyStopping,
    pub taille_cellule_cm: u32,
    pub seuil_invasif_cm2: i64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct EarlyStopping {
    pub patience: usize,
    pub seuil_amelioration: f64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Chemins {
    pub plantes: String,
    pub bacs: String,
    pub interactions: String,
}

/// Toutes les données chargées depuis `data/`, prêtes pour le moteur.
pub type DonneesChargees = (Vec<Plante>, Vec<Bac>, Vec<Interaction>, Config);

// --- Erreurs ---
#[derive(Error, Debug)]
pub enum CliError {
    #[error("Fichier introuvable : {chemin}")]
    FileNotFound { chemin: String },

    #[error("JSON invalide dans {chemin} : {cause}")]
    InvalidJson { chemin: String, cause: String },

    #[error("Référence invalide : {0} (non trouvée dans les données)")]
    InvalidReference(String),

    #[error("Paire en doublon dans interactions.json : ({0}, {1})")]
    DuplicatePair(String, String),
}

// --- Parsing ---
#[derive(clap::Parser, Debug)]
#[command(name = "ga-potager-cli")]
#[command(version = "0.1.0")]
pub struct Args {
    #[arg(short, long, default_value = "data/")]
    pub data_dir: PathBuf,

    #[arg(short, long, default_value = "data/config.json")]
    pub config_path: PathBuf,

    /// Surcharge la seed du config.json (reproductibilité des runs)
    #[arg(long)]
    pub seed: Option<u64>,
}

/// Charge et désérialise un fichier JSON vers le type `T`.
fn load_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, CliError> {
    let content = fs::read_to_string(path)
        .map_err(|_| CliError::FileNotFound { chemin: path.display().to_string() })?;

    serde_json::from_str(&content).map_err(|e| CliError::InvalidJson {
        chemin: path.display().to_string(),
        cause: e.to_string(),
    })
}

/// Valide les références croisées et les doublons dans les interactions.
pub fn validate_references(
    plantes: &[Plante],
    interactions: &[Interaction],
) -> Result<(), CliError> {
    let plant_ids: Vec<&str> = plantes.iter().map(|p| p.id.as_str()).collect();

    for interaction in interactions {
        if !plant_ids.contains(&interaction.plante_a.as_str()) {
            return Err(CliError::InvalidReference(interaction.plante_a.clone()));
        }
        if !plant_ids.contains(&interaction.plante_b.as_str()) {
            return Err(CliError::InvalidReference(interaction.plante_b.clone()));
        }
    }

    // Détecte les doublons (symétrie implicite : (A,B) == (B,A))
    let mut seen_pairs = std::collections::HashSet::new();
    for interaction in interactions {
        let pair = if interaction.plante_a < interaction.plante_b {
            (interaction.plante_a.as_str(), interaction.plante_b.as_str())
        } else {
            (interaction.plante_b.as_str(), interaction.plante_a.as_str())
        };
        if !seen_pairs.insert(pair) {
            return Err(CliError::DuplicatePair(
                interaction.plante_a.clone(),
                interaction.plante_b.clone(),
            ));
        }
    }

    Ok(())
}

/// Charge et valide toutes les données depuis `data_dir`.
pub fn load_and_validate_data(
    data_dir: &Path,
    config_path: &Path,
) -> Result<DonneesChargees, CliError> {
    let plantes = load_json::<Vec<Plante>>(&data_dir.join("plantes.json"))?;
    let bacs = load_json::<Vec<Bac>>(&data_dir.join("bacs.json"))?;
    let interactions = load_json::<Vec<Interaction>>(&data_dir.join("interactions.json"))?;
    let config = load_json::<Config>(config_path)?;

    validate_references(&plantes, &interactions)?;

    Ok((plantes, bacs, interactions, config))
}
