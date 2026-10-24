// ga-plantes/src/fitness/eval.rs
use crate::cli::Interaction;
use crate::models::{Combinaison, InteractionType, Intensite};

/// Contexte immuable : interactions chargées une fois au démarrage.
pub struct Evaluateur {
    pub interactions: Vec<Interaction>,
}

impl Evaluateur {
    pub fn new(interactions: Vec<Interaction>) -> Self {
        Self { interactions }
    }

    /// Évalue une combinaison. Plus haut = mieux.
    pub fn evaluer(&self, combinaison: &Combinaison) -> f64 {
        // 1. Rendement brut : somme des valeurs de rendement des plantes
        let rendement: f64 = combinaison
            .plantes
            .iter()
            .map(|p| p.rendement.valeur)
            .sum();

        // 2. Interactions entre paires de plantes du bac
        let interactions = self.score_interactions(&combinaison.plantes
            .iter()
            .map(|p| p.id.clone())
            .collect::<Vec<_>>());

        // 3. Occupation : on récompense l'usage de l'espace (0 pour un bac vide)
        let surface_utilisee: f64 = combinaison
            .plantes
            .iter()
            .map(|p| (p.espacement_min_cm * p.espacement_min_cm) as f64)
            .sum();
        let surface_utile = combinaison.bac.surface_utile_cm2() as f64;
        let occupation = surface_utilisee / surface_utile; // ratio [0, 1]

        // 4. Combinaison pondérée — les poids sont réglables
        rendement * 5.0
            + interactions * 7.0
            + occupation * 10.0
    }

    /// Compagnonnage > 0, compétition < 0, pondéré par intensité.
    /// **Note** : `Forte` pondérée à 1.5 (au lieu de 2.0) temporairement —
    /// les données actuelles n'ont pas encore de forte confiance, on garde
    /// de la diversité dans les solutions en attendant de vraies données.
    fn score_interactions(&self, ids: &[String]) -> f64 {
        let mut score = 0.0;
        for it in &self.interactions {
            let a_present = ids.contains(&it.plante_a);
            let b_present = ids.contains(&it.plante_b);
            if !a_present || !b_present {
                continue; // l'interaction ne s'applique pas
            }
            let poids_intensite = match it.intensite {
                Intensite::Faible => 0.5,
                Intensite::Moyenne => 1.0,
                Intensite::Forte => 1.5,
            };
            score += match it.r#type {
                InteractionType::Compagnonnage => poids_intensite,
                InteractionType::Neutre => 0.0,
                _ => -poids_intensite, // compétition (racine, eau, lumière)
            };
            // TODO (M2-c+) : pondérer par la confiance de la donnée
        }
        score
    }
}
