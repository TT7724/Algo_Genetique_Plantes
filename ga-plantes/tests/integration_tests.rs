//! Tests d'intégration sur les VRAIES données de `data/`.
//! Le répertoire courant à l'exécution des tests est la crate
//! (`ga-plantes/`), d'où les chemins relatifs en `../data/`.

use ga_plantes::cli::load_and_validate_data;
use ga_plantes::ga::validation::{est_valide, ValidationError};
use ga_plantes::models::{Bac, Combinaison, DimensionsCm, Exposition, Protection, Saison, TempMoyenneC};
use std::path::Path;

fn charger_donnees() -> (Vec<ga_plantes::models::Plante>, Vec<ga_plantes::models::Bac>, Vec<ga_plantes::cli::Interaction>, ga_plantes::cli::Config) {
    let data_dir = Path::new("../data/");
    let config_path = Path::new("../data/config.json");
    load_and_validate_data(data_dir, config_path).expect("Échec du chargement des données réelles")
}

#[test]
fn test_load_and_validate_data_reelles() {
    charger_donnees();
}

#[test]
fn test_un_seul_bac_de_reference() {
    // Le dataset v1 ne contient qu'un bac : l'environnement de référence.
    let (_, bacs, _, _) = charger_donnees();
    assert_eq!(bacs.len(), 1);
    assert_eq!(bacs[0].id, "reference");
}

#[test]
fn test_courgette_dans_bac_de_reference_invalide() {
    // Le bac de référence a 40 cm de terre : la carotte (racines 30 cm)
    // y est valide — c'est la courgette (racines 50 cm) qui ne tient pas.
    let (plantes, bacs, _, config) = charger_donnees();
    let bac = bacs.iter().find(|b| b.id == "reference").expect("Bac 'reference' introuvable");
    let courgette = plantes.iter().find(|p| p.id == "courgette").expect("Plante 'courgette' introuvable");

    let combinaison = Combinaison::new(vec![courgette.clone()], bac.clone());
    let result = est_valide(&combinaison, &config);

    assert!(
        matches!(result, Err(ValidationError::RacinesTropProfondes(50, 40))),
        "La courgette (50 cm de racines) ne devrait pas tenir dans le bac de référence (40 cm de terre)"
    );
}

#[test]
fn test_bac_draine_nest_pas_hydroponique() {
    // Le fix du discriminuteur : le bac de référence est drainé MAIS a
    // de la terre — la contrainte de profondeur doit s'appliquer.
    // (Un simple drainage ne doit pas neutraliser les contraintes.)
    let (plantes, bacs, _, config) = charger_donnees();
    let bac = bacs.iter().find(|b| b.id == "reference").expect("Bac 'reference' introuvable");
    assert!(bac.drainage_eau, "le bac de référence est drainé");
    assert!(!bac.substrats.is_empty(), "mais il contient de la terre");

    let courgette = plantes.iter().find(|p| p.id == "courgette").expect("Plante 'courgette' introuvable");
    let combinaison = Combinaison::new(vec![courgette.clone()], bac.clone());
    assert!(est_valide(&combinaison, &config).is_err());
}

#[test]
fn test_laitue_en_bac_hydroponique_valide() {
    // Valide le fix hydroponique : un bac hydro (0 cm de terre, aucun
    // substrat) doit pouvoir accueillir une plante à enracinement eau.
    // Bac construit en fixture : le test vise la LOGIQUE de validation.
    let (plantes, _, _, config) = charger_donnees();
    let laitue = plantes.iter().find(|p| p.id == "laitue").expect("Plante 'laitue' introuvable");

    let bac_hydro = Bac {
        id: "hydro_fixture".to_string(),
        nom: "Bac hydroponique (fixture)".to_string(),
        dimensions_cm: DimensionsCm { longueur: 150, largeur: 100, hauteur: 25 },
        hauteur_terre_cm: 0,
        marge_bord_cm: 2,
        protection: Protection::SerreChauffee,
        substrats: vec![],
        compost_disponible: false,
        drainage_eau: true,
        exposition: Exposition::MiOmbre,
        saison_cible: Saison::Printemps,
        temp_moyenne_c: TempMoyenneC { min: 10, max: 20 },
    };

    let combinaison = Combinaison::new(vec![laitue.clone()], bac_hydro);
    let result = est_valide(&combinaison, &config);

    assert!(result.is_ok(), "La laitue devrait être valide dans un bac hydroponique : {result:?}");
}
