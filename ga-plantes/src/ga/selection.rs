use crate::models::Combinaison;
use rand::Rng;
use rand::rngs::StdRng;
use thiserror::Error;

/// Erreurs possibles lors de la sélection.
#[derive(Error, Debug)]
pub enum SelectionError {
    #[error("Population vide : impossible de sélectionner un individu")]
    PopulationVide,

    #[error("Taille du tournoi invalide : {0} (doit être > 0 et <= taille de la population)")]
    TailleTournoiInvalide(usize),
}

/// Sélectionne un individu par tournoi parmi la population.
/// - **Scores précalculés** : la fitness est calculée **une fois par génération**
///   par `evolution.rs` ; la sélection lit dans le slice, elle ne recalcule rien.
/// - **Tirage avec remise** : un même individu peut être tiré plusieurs fois.
///
/// # Arguments
/// * `population` - Liste des combinaisons candidates.
/// * `taille_tournoi` - Nombre d'individus tirés aléatoirement pour le tournoi.
/// * `scores` - Fitness précalculées, alignées avec `population` (même longueur).
/// * `rng` - Générateur aléatoire mutable (graine semée dans `evolution.rs`).
pub fn selection_tournoi<'a>(
    population: &'a [Combinaison],
    taille_tournoi: usize,
    scores: &[f64],
    rng: &mut StdRng,
) -> Result<&'a Combinaison, SelectionError> {
    if population.is_empty() {
        return Err(SelectionError::PopulationVide);
    }
    if scores.len() != population.len() {
        // Invariant défensif : les scores doivent être alignés sur la population
        return Err(SelectionError::PopulationVide);
    }
    if taille_tournoi == 0 || taille_tournoi > population.len() {
        return Err(SelectionError::TailleTournoiInvalide(taille_tournoi));
    }

    // Tirer le premier individu pour initialiser le champion
    let mut best_idx = rng.gen_range(0..population.len());
    let mut best_score = scores[best_idx];

    // Tirer les k-1 autres individus et comparer
    for _ in 1..taille_tournoi {
        let idx = rng.gen_range(0..population.len());
        if scores[idx] > best_score {
            best_idx = idx;
            best_score = scores[idx];
        }
    }

    Ok(&population[best_idx])
}
