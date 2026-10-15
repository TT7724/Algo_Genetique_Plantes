use crate::cli::Config;
use crate::ga::validation::reparer;
use crate::models::{Combinaison, Plante};
use rand::Rng;
use rand::rngs::StdRng;
use thiserror::Error;

/// Erreurs possibles lors de la mutation.
#[derive(Error, Debug)]
pub enum MutationError {
    #[error("Pool de plantes vide : impossible d'ajouter une plante")]
    PoolVide,

    #[error("Réparation impossible après mutation")]
    ReparationImpossible,
}

/// Effectue une mutation sur une combinaison.
/// **Opérations possibles** (choisies aléatoirement à 50/50) :
/// - Retirer une plante aléatoire de la combinaison.
/// - Ajouter une plante aléatoire du pool, **absente de la combinaison**.
///
/// **Note** : "Déplacer" n'est pas implémenté car `Combinaison` est une liste
/// (pas une grille avec positions) — la diversité spatiale viendra en M2-c.
///
/// **Taux** : contrôlé par `config.algorithme.taux_mutation` (probabilité
/// d'appliquer une mutation).
/// **Réparation** : appelée systématiquement après mutation pour garantir la validité.
pub fn muter(
    combinaison: Combinaison,
    pool: &[Plante],
    config: &Config,
    rng: &mut StdRng,
) -> Result<Combinaison, MutationError> {
    // 1. Décide si on applique une mutation (selon taux_mutation)
    if !rng.gen_bool(config.algorithme.taux_mutation) {
        return Ok(combinaison); // Pas de mutation
    }

    let mut new_combinaison = combinaison;

    // 2. Choisit l'opération : retirer (true) ou ajouter (false)
    let operation = if !new_combinaison.plantes.is_empty() && !pool.is_empty() {
        rng.gen_bool(0.5)
    } else if !new_combinaison.plantes.is_empty() {
        true // Force retirer si pool vide
    } else if !pool.is_empty() {
        false // Force ajouter si combinaison vide
    } else {
        return Err(MutationError::PoolVide);
    };

    if operation {
        // Retirer une plante aléatoire
        let idx = rng.gen_range(0..new_combinaison.plantes.len());
        new_combinaison.plantes.remove(idx);
    } else {
        // Ajouter une plante du pool non déjà présente
        let candidates: Vec<&Plante> = pool
            .iter()
            .filter(|p| !new_combinaison.plantes.iter().any(|existing| existing.id == p.id))
            .collect();

        if candidates.is_empty() {
            // Tout le pool est déjà dans la combinaison : retire à la place
            if !new_combinaison.plantes.is_empty() {
                let idx = rng.gen_range(0..new_combinaison.plantes.len());
                new_combinaison.plantes.remove(idx);
            }
        } else {
            let idx = rng.gen_range(0..candidates.len());
            new_combinaison.plantes.push(candidates[idx].clone());
        }
    }

    // 3. Répare la combinaison après mutation
    reparer(new_combinaison, config, rng).map_err(|_| MutationError::ReparationImpossible)
}
