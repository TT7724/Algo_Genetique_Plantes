use crate::cli::Config;
use crate::ga::crossover::{crossover, CrossoverError};
use crate::ga::mutation::{muter, MutationError};
use crate::ga::selection::{selection_tournoi, SelectionError};
use crate::models::{Combinaison, Plante};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use thiserror::Error;

/// Résultat de l'évolution : meilleur individu + nombre de générations exécutées.
#[derive(Debug, Clone)]
pub struct EvolutionResult {
    pub meilleur_individu: Combinaison,
    pub generations: usize,
}

/// Erreurs possibles lors de l'évolution.
#[derive(Error, Debug)]
pub enum EvolutionError {
    #[error("Population initiale vide")]
    PopulationVide,

    #[error("Erreur de sélection : {0}")]
    Selection(#[from] SelectionError),

    #[error("Erreur de croisement : {0}")]
    Croisement(#[from] CrossoverError),

    #[error("Erreur de mutation : {0}")]
    Mutation(#[from] MutationError),
}

/// Exécute l'algorithme génétique.
///
/// **Flux** :
/// 1. Initialise le RNG avec `config.algorithme.seed` — **une seule fois**,
///    le même générateur traverse sélection, croisement et mutation.
/// 2. Calcule les scores **une fois par génération** ; les tournois lisent
///    dans ce slice (aucun recalcul de fitness).
/// 3. **Élitisme** : les `config.algorithme.elitisme` meilleurs passent
///    intacts à la génération suivante.
/// 4. **Croisement probabiliste** : appliqué selon `taux_croisement` ;
///    sinon les parents sont recopiés.
/// 5. **Early stopping** : si aucune amélioration supérieure à
///    `early_stopping.seuil_amelioration` pendant `early_stopping.patience`
///    générations, on s'arrête et on retourne le meilleur individu jamais vu.
/// 6. **Aucune logique de fitness** dans le moteur : `score_fn` est injectée.
pub fn evoluer<F>(
    population_initiale: Vec<Combinaison>,
    pool: &[Plante],
    config: &Config,
    taille_tournoi: usize,
    score_fn: F,
) -> Result<EvolutionResult, EvolutionError>
where
    F: Fn(&Combinaison) -> f64,
{
    if population_initiale.is_empty() {
        return Err(EvolutionError::PopulationVide);
    }

    let algo = &config.algorithme;
    let mut rng = StdRng::seed_from_u64(algo.seed);

    let mut population = population_initiale;
    let elitisme = algo.elitisme;
    let patience_max = algo.early_stopping.patience;
    let seuil_amelioration = algo.early_stopping.seuil_amelioration;

    let mut meilleur_score_global = f64::NEG_INFINITY;
    let mut meilleur_individu_global: Option<Combinaison> = None;
    let mut patience = 0usize;
    let mut generations_execuees = 0usize;

    for generation in 0..algo.generations_max {
        generations_execuees = generation + 1;

        // a. Scores calculés UNE FOIS par génération
        let scores: Vec<f64> = population.iter().map(&score_fn).collect();

        // b. Meilleur individu de la génération (tri d'indices, sans dépendance externe)
        let mut indices: Vec<usize> = (0..population.len()).collect();
        indices.sort_by(|&a, &b| {
            scores[b]
                .partial_cmp(&scores[a])
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let best_idx = indices[0];
        let current_best_score = scores[best_idx];

        // c. Mise à jour du meilleur global (toute amélioration compte)
        if current_best_score > meilleur_score_global {
            meilleur_score_global = current_best_score;
            meilleur_individu_global = Some(population[best_idx].clone());
        }

        // d. Early stopping : stagnation au-delà du seuil d'amélioration
        let amelioration = current_best_score - meilleur_score_global;
        if amelioration > seuil_amelioration || meilleur_score_global == current_best_score && generation == 0 {
            patience = 0;
        } else {
            patience += 1;
            if patience >= patience_max {
                return Ok(EvolutionResult {
                    meilleur_individu: meilleur_individu_global.expect("initialisé à la génération 0"),
                    generations: generations_execuees,
                });
            }
        }

        // e. Élitisme : les `elitisme` meilleurs passent intacts
        let mut nouvelle_population: Vec<Combinaison> = indices
            .iter()
            .take(elitisme)
            .map(|&i| population[i].clone())
            .collect();

        // f. Croisement probabiliste + mutation pour remplir la population
        while nouvelle_population.len() < population.len() {
            let parent1 =
                selection_tournoi(&population, taille_tournoi, &scores, &mut rng)?.clone();
            let parent2 =
                selection_tournoi(&population, taille_tournoi, &scores, &mut rng)?.clone();

            let (enfant1, enfant2) = if rng.gen_bool(algo.taux_croisement) {
                crossover(&parent1, &parent2, config, &mut rng)?
            } else {
                // Pas de croisement : les parents sont recopiés
                (parent1.clone(), parent2.clone())
            };

            let enfant1 = muter(enfant1, pool, config, &mut rng)?;
            nouvelle_population.push(enfant1);

            if nouvelle_population.len() < population.len() {
                let enfant2 = muter(enfant2, pool, config, &mut rng)?;
                nouvelle_population.push(enfant2);
            }
        }

        // g. La génération suivante remplace la précédente
        population = nouvelle_population;
    }

    Ok(EvolutionResult {
        meilleur_individu: meilleur_individu_global.expect("initialisé à la génération 0"),
        generations: generations_execuees,
    })
}
