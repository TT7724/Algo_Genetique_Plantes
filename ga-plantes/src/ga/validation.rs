use crate::cli::Config;
use crate::models::{Combinaison, Saison, TauxCroissance};
use thiserror::Error;

/// Erreurs de validation des contraintes dures.
#[derive(Error, Debug, PartialEq)]
pub enum ValidationError {
    #[error("Surface insuffisante : {0} cm² requis, {1} cm² disponible (approximation conservative : carré d'espacement)")]
    SurfaceInsuffisante(i64, i64),

    #[error("Profondeur des racines trop grande : {0} cm (plante) > {1} cm (bac)")]
    RacinesTropProfondes(u32, u32),

    #[error("Plante {0} nécessite un enracinement dans l'eau, mais le bac ne le permet pas")]
    EnracinementEauIncompatible(String),

    #[error("Substrat incompatible : la plante {0} nécessite {1:?}, le bac offre {2:?}")]
    SubstratIncompatible(String, Vec<String>, Vec<String>),

    #[error("Saison incompatible : la plante {0} accepte {1:?}, le bac est en {2:?}")]
    SaisonIncompatible(String, Vec<Saison>, Saison),

    #[error("Température incompatible : la plante {0} survit entre [{1}-{2}°C], le bac offre [{3}-{4}°C] (contrainte de survie, pas de confort)")]
    TemperatureIncompatible(String, i32, i32, i32, i32),

    #[error("Plante invasive {0} interdite : surface bac {1} cm² < seuil {2} cm²")]
    InvasiveInterdite(String, i64, i64),
}

/// Vérifie que la combinaison respecte **toutes** les contraintes dures.
/// **Note** : un bac vide est autorisé (la fitness s'en chargera).
pub fn est_valide(combinaison: &Combinaison, config: &Config) -> Result<(), ValidationError> {
    // 1. Surface totale des plantes <= surface utile du bac
    // Approximation conservative : chaque plante occupe un carré de côté = espacement_min_cm
    let surface_totale: i64 = combinaison
        .plantes
        .iter()
        .map(|p| (p.espacement_min_cm * p.espacement_min_cm) as i64)
        .sum();
    if surface_totale > combinaison.bac.surface_utile_cm2() {
        return Err(ValidationError::SurfaceInsuffisante(
            surface_totale,
            combinaison.bac.surface_utile_cm2(),
        ));
    }

    // 2. Profondeur des racines <= hauteur de terre du bac
    for plante in &combinaison.plantes {
        if plante.profondeur_racines_cm > combinaison.bac.hauteur_terre_cm {
            return Err(ValidationError::RacinesTropProfondes(
                plante.profondeur_racines_cm,
                combinaison.bac.hauteur_terre_cm,
            ));
        }
    }

    // 3. Enracinement dans l'eau : plante hydro exigerait un bac hydroponique
    for plante in &combinaison.plantes {
        if plante.enracinement_eau && !combinaison.bac.drainage_eau {
            return Err(ValidationError::EnracinementEauIncompatible(plante.id.clone()));
        }
    }

    // 4. Substrats : au moins un substrat commun entre plante et bac
    for plante in &combinaison.plantes {
        let substrats_plante = &plante.substrats_compatibles;
        let substrats_bac = &combinaison.bac.substrats;
        if !substrats_plante.iter().any(|s| substrats_bac.contains(s)) {
            return Err(ValidationError::SubstratIncompatible(
                plante.id.clone(),
                substrats_plante.clone(),
                substrats_bac.clone(),
            ));
        }
    }

    // 5. Saison : la saison cible du bac doit être acceptée par la plante
    for plante in &combinaison.plantes {
        if !plante.saisons.contains(&combinaison.bac.saison_cible) {
            return Err(ValidationError::SaisonIncompatible(
                plante.id.clone(),
                plante.saisons.clone(),
                combinaison.bac.saison_cible,
            ));
        }
    }

    // 6. Température (contrainte de survie, pas de confort) : plages disjointes = invalide
    for plante in &combinaison.plantes {
        let temp_plante_min = plante.temperature_c.min_survie;
        let temp_plante_max = plante.temperature_c.max_survie;
        let temp_bac_min = combinaison.bac.temp_moyenne_c.min;
        let temp_bac_max = combinaison.bac.temp_moyenne_c.max;

        if temp_plante_max < temp_bac_min || temp_plante_min > temp_bac_max {
            return Err(ValidationError::TemperatureIncompatible(
                plante.id.clone(),
                temp_plante_min,
                temp_plante_max,
                temp_bac_min,
                temp_bac_max,
            ));
        }
    }

    // 7. Contrainte invasive : interdite si la surface utile est sous le seuil
    for plante in &combinaison.plantes {
        if plante.taux_croissance == TauxCroissance::Invasive
            && combinaison.bac.surface_utile_cm2() < config.algorithme.seuil_invasif_cm2
        {
            return Err(ValidationError::InvasiveInterdite(
                plante.id.clone(),
                combinaison.bac.surface_utile_cm2(),
                config.algorithme.seuil_invasif_cm2,
            ));
        }
    }

    Ok(())
}
