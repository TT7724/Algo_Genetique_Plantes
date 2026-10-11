use crate::models::{Bac, Plante};

/// Représente une combinaison de plantes dans un bac.
/// **Modèle pur** : ne contient aucune logique métier.
#[derive(Debug, Clone)]
pub struct Combinaison {
    pub plantes: Vec<Plante>,
    pub bac: Bac,
}

impl Combinaison {
    /// Crée une nouvelle combinaison.
    pub fn new(plantes: Vec<Plante>, bac: Bac) -> Self {
        Self { plantes, bac }
    }
}
