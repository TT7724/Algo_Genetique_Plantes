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
    // Le test ultime du parsing : les 4 vrais fichiers doivent se charger,
    // se désérialiser et passer la validation des références croisées.
    charger_donnees();
}

#[test]
fn test_un_seul_bac_de_reference() {
    // Le dataset v1 ne contient qu'un bac : l'environnement de référence.
    // (Anciennement 3 bacs ; l'AG optimise pour un bac donné — v1 du plan.)
    let (_, bacs, _, _) = charger_donnees();
    assert_eq!(bacs.len(), 1);
    assert_eq!(bacs[0].id, "reference");
}

#[test]
fn test_courgette_dans_bac_de_reference_invalide() {
    // Le bac de référence a 40 cm de terre depuis le passage au mono-bac :
    // la carotte (racines 30 cm) y est désormais VALIDE — le test
    // d'invalidité migre vers la courgette (racines 50 cm).
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
fn test_basilic_en_bac_hydroponique_valide() {
    // Valide le fix hydroponique : un bac hydro (0 cm de terre, aucun
    // substrat) doit pouvoir accueillir une plante à enracinement eau.
    // Bac construit en fixture : le test vise la LOGIQUE de validation,
    // pas le contenu de bacs.json (qui ne contient plus qu'un bac terre).
    let (plantes, _, _, config) = charger_donnees();
    let basilic = plantes.iter().find(|p| p.id == "basilic").expect("Plante 'basilic' introuvable");

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
        exposition: Exposition::PleinSoleil,
        saison_cible: Saison::Ete,
        temp_moyenne_c: TempMoyenneC { min: 20, max: 30 },
    };

    let combinaison = Combinaison::new(vec![basilic.clone()], bac_hydro);
    let result = est_valide(&combinaison, &config);

    assert!(result.is_ok(), "Le basilic devrait être valide dans un bac hydroponique : {result:?}");
}
