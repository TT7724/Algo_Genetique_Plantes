//! Tests d'intégration sur les VRAIES données de `data/`.
//! Le répertoire courant à l'exécution des tests est la crate
//! (`ga-plantes/`), d'où les chemins relatifs en `../data/`.

use ga_plantes::cli::load_and_validate_data;
use ga_plantes::ga::validation::{est_valide, ValidationError};
use ga_plantes::models::Combinaison;
use std::path::Path;

fn charger_donnees() -> (Vec<ga_plantes::models::Plante>, Vec<ga_plantes::models::Bac>, Vec<ga_plantes::cli::Interaction>, ga_plantes::cli::Config) {
    let data_dir = Path::new("../data/");
    let config_path = Path::new("../data/config.json");
    load_and_validate_data(data_dir, config_path).expect("Échec du chargement des données réelles")
}

#[test]
fn test_load_and_validate_data_reelles() {
    // Le test ultime du parsing : les 4 vrais fichiers doivent se charger,
    // se désérialiser et passer la validation des références croisées.
    charger_donnees();
}

#[test]
fn test_carotte_dans_bac_contraint_invalide() {
    // Scénario construit dès M1 : le bac contraint (15 cm de terre)
    // ne peut pas accueillir la carotte (racines 30 cm).
    let (plantes, bacs, _, config) = charger_donnees();
    let bac_contraint = bacs.iter().find(|b| b.id == "contraint").expect("Bac 'contraint' introuvable");
    let carotte = plantes.iter().find(|p| p.id == "carotte").expect("Plante 'carotte' introuvable");

    let combinaison = Combinaison::new(vec![carotte.clone()], bac_contraint.clone());
    let result = est_valide(&combinaison, &config);

    assert!(
        matches!(result, Err(ValidationError::RacinesTropProfondes(30, 15))),
        "La carotte (30 cm de racines) ne devrait pas tenir dans le bac contraint (15 cm de terre)"
    );
}

#[test]
fn test_basilic_dans_bac_hydro_valide() {
    // Valide le fix hydroponique : un bac hydro (0 cm de terre, aucun
    // substrat) doit pouvoir accueillir une plante à enracinement eau.
    let (plantes, bacs, _, config) = charger_donnees();
    let bac_hydro = bacs.iter().find(|b| b.id == "hydro").expect("Bac 'hydro' introuvable");
    let basilic = plantes.iter().find(|p| p.id == "basilic").expect("Plante 'basilic' introuvable");

    let combinaison = Combinaison::new(vec![basilic.clone()], bac_hydro.clone());
    let result = est_valide(&combinaison, &config);

    assert!(result.is_ok(), "Le basilic devrait être valide dans le bac hydroponique : {result:?}");
}
