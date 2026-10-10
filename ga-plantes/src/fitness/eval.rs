//! MODULE ISOLÉ — la vraie fitness est écrite par l'utilisateur.
//!
//! Structure cible (5 critères normalisés sur [0,1], pondérés) :
//!   fitness = 0.30·rendement + 0.20·sante + 0.15·resistance_ravageurs
//!           + 0.15·ressources + 0.20·diversite
//! Pénalités souples : saison/températures (atténuées par la protection serre),
//! invasives en petit bac, incohérences d'eau/nutriments.
//!
//! TODO (utilisateur) : implémenter la vraie fitness ici.
//! Le moteur ne doit JAMAIS contenir de logique de fitness.

/// Fitness factice pour les tests du moteur. Ne pas utiliser en production.
pub fn fitness_stub(individu: &[String]) -> f32 {
    // Pondération arbitraire : rendement total ~ nb de plantes.
    individu.len() as f32
}
