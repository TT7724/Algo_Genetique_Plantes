use crate::cli::Config;
use crate::ga::crossover::{crossover, CrossoverError};
use crate::ga::mutation::{muter, MutationError};
use crate::ga::selection::{selection_tournoi, SelectionError};
use crate::models::{Combinaison, Plante};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::collections::HashMap;
use thiserror::Error;

/// Statistiques d'une génération — pour les courbes de convergence
/// et de composition dans le notebook (orchestrateur M3).
#[derive(Debug, Clone, serde::Serialize)]
pub struct HistoriqueGeneration {
    pub generation: usize,
    pub fitness_min: f64,
    pub fitness_moyenne: f64,
    pub fitness_max: f64,
    /// Nombre d'individus contenant chaque plante (id → occurrences).
    /// Seules les plantes présentes au moins une fois figurent ici,
    /// triées par id pour un JSON déterministe.
    pub presence_plantes: Vec<(String, usize)>,
}

/// Résultat de l'évolution : meilleur individu + nombre de générations exécutées.
#[derive(Debug, Clone)]
pub struct EvolutionResult {
    pub meilleur_individu: Combinaison,
    pub generations: usize,
    /// Population de la dernière génération — pour analyser la diversité
    /// et comparer les bonnes/mauvaises propositions (orchestrateur M3).
    pub population_finale: Vec<Combinaison>,
    /// Statistiques par génération (fitness min/moy/max, présence des plantes).
    pub historique: Vec<HistoriqueGeneration>,
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
/// 6. **Historique** : statistiques (fitness min/moy/max, présence des plantes)
///    collectées à chaque génération — pure mesure, aucun effet sur le moteur.
/// 7. **Aucune logique de fitness** dans le moteur : `score_fn` est injectée.
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
    let mut historique: Vec<HistoriqueGeneration> = Vec::new();

    for generation in 0..algo.generations_max {
        generations_execuees = generation + 1;

        // a. Scores calculés UNE FOIS par génération
        let scores: Vec<f64> = population.iter().map(&score_fn).collect();

        // a'. Historique : statistiques de la génération courante (pure mesure)
        let fitness_min = scores.iter().cloned().fold(f64::INFINITY, f64::min);
        let fitness_max = scores.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let fitness_moyenne = scores.iter().sum::<f64>() / scores.len() as f64;
        let mut plante_counts: HashMap<String, usize> = HashMap::new();
        for combinaison in &population {
            for plante in &combinaison.plantes {
                *plante_counts.entry(plante.id.clone()).or_insert(0) += 1;
            }
        }
        let mut presence_plantes: Vec<(String, usize)> = plante_counts.into_iter().collect();
        presence_plantes.sort_by(|a, b| a.0.cmp(&b.0));
        historique.push(HistoriqueGeneration {
            generation,
            fitness_min,
            fitness_moyenne,
            fitness_max,
            presence_plantes,
        });

        // b. Meilleur individu de la génération (tri d'indices, sans dépendance externe)
        let mut indices: Vec<usize> = (0..population.len()).collect();
        indices.sort_by(|&a, &b| {
            scores[b]
                .partial_cmp(&scores[a])
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let best_idx = indices[0];
        let current_best_score = scores[best_idx];

        // c. Early stopping : l'amélioration se mesure AVANT la mise à jour du meilleur global
        let amelioration = current_best_score - meilleur_score_global;
        if amelioration > seuil_amelioration {
            patience = 0;
        } else {
            patience += 1;
            if patience >= patience_max {
                return Ok(EvolutionResult {
                    meilleur_individu: meilleur_individu_global.expect("initialisé à la génération 0"),
                    generations: generations_execuees,
                    population_finale: population,
                    historique,
                });
            }
        }

        // d. Mise à jour du meilleur global (toute amélioration compte)
        if current_best_score > meilleur_score_global {
            meilleur_score_global = current_best_score;
            meilleur_individu_global = Some(population[best_idx].clone());
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
        population_finale: population,
        historique,
    })
}
