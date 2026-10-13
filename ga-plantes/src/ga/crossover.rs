use crate::cli::Config;
use crate::ga::validation::est_valide;
use crate::models::{Combinaison, Plante};
use rand::Rng;
use rand::rngs::StdRng;
use thiserror::Error;

/// Erreurs possibles lors du croisement.
#[derive(Error, Debug)]
pub enum CrossoverError {
    #[error("Parents vides : impossible de croiser")]
    ParentsVides,

    #[error("Réparation impossible : aucune combinaison valide trouvée après {0} tentatives")]
    ReparationImpossible(usize),
}

/// Effectue un croisement entre deux combinaisons (parents).
/// - **Croisement 2D** :
///   - **Plantes** : mélange uniforme (chaque plante a 50% de chance de venir
///     de parent1 ou parent2), **dédupliqué par id** (pas de doublon de plante).
///   - **Bac** : choix aléatoire entre les bacs des deux parents.
/// - **Réparation** : si l'enfant est invalide, supprime aléatoirement des
///   plantes jusqu'à validité (un bac vide est valide par définition).
///
/// **Limite connue** : la réparation ne change jamais de bac — un enfant qui
/// hérite d'un bac incompatible verra ses plantes supprimées une à une.
///
/// # Arguments
/// * `parent1` - Première combinaison parent.
/// * `parent2` - Deuxième combinaison parent.
/// * `config` - Configuration (pour la réparation, ex: `seuil_invasif_cm2`).
/// * `rng` - Générateur aléatoire mutable.
pub fn crossover(
    parent1: &Combinaison,
    parent2: &Combinaison,
    config: &Config,
    rng: &mut StdRng,
) -> Result<(Combinaison, Combinaison), CrossoverError> {
    if parent1.plantes.is_empty() && parent2.plantes.is_empty() {
        return Err(CrossoverError::ParentsVides);
    }

    // Croisement uniforme sur les plantes (dédupliqué par id)
    let enfant1_plantes = croisement_uniforme(&parent1.plantes, &parent2.plantes, rng);
    let enfant2_plantes = croisement_uniforme(&parent2.plantes, &parent1.plantes, rng);

    // Choix aléatoire du bac parmi les deux parents, pour chaque enfant
    let enfant1_bac = if rng.gen_bool(0.5) {
        parent1.bac.clone()
    } else {
        parent2.bac.clone()
    };
    let enfant2_bac = if rng.gen_bool(0.5) {
        parent1.bac.clone()
    } else {
        parent2.bac.clone()
    };

    // Réparation des enfants si nécessaire
    let enfant1 = reparer(Combinaison::new(enfant1_plantes, enfant1_bac), config, rng)?;
    let enfant2 = reparer(Combinaison::new(enfant2_plantes, enfant2_bac), config, rng)?;

    Ok((enfant1, enfant2))
}

/// Croisement uniforme entre deux listes de plantes, dédupliqué par id :
/// chaque position est tirée d'un parent au hasard, et une plante dont
/// l'id est déjà présent dans l'enfant est ignorée (pas de doublon).
fn croisement_uniforme(parent1: &[Plante], parent2: &[Plante], rng: &mut StdRng) -> Vec<Plante> {
    let mut enfant: Vec<Plante> = Vec::new();
    let max_len = parent1.len().max(parent2.len());

    for i in 0..max_len {
        let candidate = if i < parent1.len() && i < parent2.len() {
            if rng.gen_bool(0.5) { &parent1[i] } else { &parent2[i] }
        } else if i < parent1.len() {
            &parent1[i]
        } else {
            &parent2[i]
        };

        // Déduplication : une plante ne peut apparaître qu'une fois
        if !enfant.iter().any(|p| p.id == candidate.id) {
            enfant.push(candidate.clone());
        }
    }

    enfant
}

/// Répare une combinaison invalide en supprimant aléatoirement des plantes.
/// 1. Si la combinaison est valide, la retourner telle quelle.
/// 2. Sinon, supprimer une plante aléatoire et réessayer (jusqu'à 100 tentatives).
/// 3. Un bac vide étant valide, la réparation aboutit toujours avant l'épuisement.
fn reparer(
    mut combinaison: Combinaison,
    config: &Config,
    rng: &mut StdRng,
) -> Result<Combinaison, CrossoverError> {
    let max_tentatives = 100;
    for _ in 0..max_tentatives {
        if est_valide(&combinaison, config).is_ok() {
            return Ok(combinaison);
        }

        if combinaison.plantes.is_empty() {
            // Plus de plantes : bac vide, valide par définition
            return Ok(combinaison);
        }

        let idx = rng.gen_range(0..combinaison.plantes.len());
        combinaison.plantes.remove(idx);
    }

    Err(CrossoverError::ReparationImpossible(max_tentatives))
}
