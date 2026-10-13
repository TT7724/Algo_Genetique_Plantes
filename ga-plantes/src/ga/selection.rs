use ga_plantes::models::Combinaison;
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
/// - **Reproductibilité** : utilise un générateur aléatoire (`rng`) passé en paramètre.
///   **Note** : La graine (`seed`) est semée une seule fois dans `evolution.rs`.
/// - **Tirage avec remise** : un même individu peut être tiré plusieurs fois dans un tournoi.
/// - **Fonction de scoring** : fournie par l'utilisateur (ex: `fitness::eval::evaluer`).
///   **Note** : Le moteur ne contient **aucune** logique de fitness.
///
/// # Arguments
/// * `population` - Liste des combinaisons candidates.
/// * `taille_tournoi` - Nombre d'individus tirés aléatoirement pour le tournoi.
/// * `rng` - Générateur aléatoire mutable (ex: `&mut StdRng`).
/// * `score_fn` - Fonction de scoring (fitness) fournie par l'utilisateur.
pub fn selection_tournoi<F>(
    population: &[Combinaison],
    taille_tournoi: usize,
    rng: &mut StdRng,
    score_fn: F,
) -> Result<&Combinaison, SelectionError>
where
    F: Fn(&Combinaison) -> f64,
{
    if population.is_empty() {
        return Err(SelectionError::PopulationVide);
    }
    if taille_tournoi == 0 || taille_tournoi > population.len() {
        return Err(SelectionError::TailleTournoiInvalide(taille_tournoi));
    }

    // Tirer le premier individu pour initialiser le champion
    let best_idx = rng.gen_range(0..population.len());
    let mut best = &population[best_idx];
    let mut best_score = score_fn(best);

    // Tirer les k-1 autres individus et comparer
    for _ in 1..taille_tournoi {
        let idx = rng.gen_range(0..population.len());
        let candidate = &population[idx];
        let candidate_score = score_fn(candidate);

        if candidate_score > best_score {
            best = candidate;
            best_score = candidate_score;
        }
    }

    Ok(best)
}
