// ga-plantes/src/fitness/eval.rs
use crate::cli::Interaction;
use crate::cli::{InteractionType, Intensite};
use crate::models::Combinaison;

/// Décomposition du score — explique POURQUOI une combinaison gagne.
#[derive(Debug, Clone)]
pub struct DetailScore {
    /// Somme des rendements (brut, avant pondération)
    pub rendement: f64,
    /// Score d'interactions (compagnonnage > 0, compétition < 0)
    pub interactions: f64,
    /// Ratio de surface occupée [0, 1]
    pub occupation: f64,
    /// Score total pondéré
    pub total: f64,
    /// Interactions qui se sont appliquées, lisibles ("tomate+basilic : compagnonnage")
    pub interactions_actives: Vec<String>,
}

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
        self.evaluer_detail(combinaison).total
    }

    /// Évalue en retournant la décomposition (pour l'explication des résultats).
    pub fn evaluer_detail(&self, combinaison: &Combinaison) -> DetailScore {
        // 1. Rendement brut : somme des valeurs de rendement des plantes
        let rendement: f64 = combinaison
            .plantes
            .iter()
            .map(|p| p.rendement.valeur as f64)
            .sum();

        // 2. Interactions entre paires de plantes du bac
        let ids: Vec<String> = combinaison.plantes.iter().map(|p| p.id.clone()).collect();
        let (interactions, interactions_actives) = self.score_interactions(&ids);

        // 3. Occupation : on récompense l'usage de l'espace (0 pour un bac vide)
        let surface_utilisee: f64 = combinaison
            .plantes
            .iter()
            .map(|p| (p.espacement_min_cm * p.espacement_min_cm) as f64)
            .sum();
        let surface_utile = combinaison.bac.surface_utile_cm2() as f64;
        let occupation = surface_utilisee / surface_utile; // ratio [0, 1]

        // 4. Combinaison pondérée — les poids sont réglables
        let total = rendement * 5.0 + interactions * 7.0 + occupation * 10.0;

        DetailScore {
            rendement,
            interactions,
            occupation,
            total,
            interactions_actives,
        }
    }

    /// Compagnonnage > 0, compétition < 0, pondéré par intensité.
    /// Retourne (score, description des interactions appliquées).
    /// **Note** : `Forte` pondérée à 1.5 (au lieu de 2.0) temporairement —
    /// les données actuelles n'ont pas encore de forte confiance, on garde
    /// de la diversité dans les solutions en attendant de vraies données.
    fn score_interactions(&self, ids: &[String]) -> (f64, Vec<String>) {
        let mut score = 0.0;
        let mut actives: Vec<String> = Vec::new();
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
            let (delta, type_nom) = match it.r#type {
                InteractionType::Compagnonnage => (poids_intensite, "compagnonnage"),
                InteractionType::Neutre => (0.0, "neutre"),
                InteractionType::CompetitionRacinaire => (-poids_intensite, "compétition racinaire"),
                InteractionType::CompetitionEau => (-poids_intensite, "compétition eau"),
                InteractionType::CompetitionLumiere => (-poids_intensite, "compétition lumière"),
            };
            score += delta;
            if delta != 0.0 {
                actives.push(format!("{}+{} : {} ({:+.1})", it.plante_a, it.plante_b, type_nom, delta));
            }
            // TODO (M2-c+) : pondérer par la confiance de la donnée
        }
        (score, actives)
    }
}
