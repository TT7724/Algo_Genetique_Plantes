use ga_plantes::cli::Config;
use ga_plantes::ga::validation::{est_valide, ValidationError};
use ga_plantes::models::{
    Bac, BesoinsNutriments, Combinaison, DimensionsCm, Exposition, Niveau, Plante, Protection,
    Rendement, Saison, TauxCroissance, TempMoyenneC, TemperatureC,
};

// --- Helpers (fixtures) ---

/// Construit une plante de test. Paramètres : l'essentiel pour les contraintes,
/// le reste est fixé à des valeurs neutres.
#[allow(clippy::too_many_arguments)]
fn plante_test(
    id: &str,
    espacement_min_cm: u32,
    profondeur_racines_cm: u32,
    enracinement_eau: bool,
    substrats: Vec<String>,
    saisons: Vec<Saison>,
    temp_min_survie: i32,
    temp_max_survie: i32,
    taux_croissance: TauxCroissance,
) -> Plante {
    Plante {
        id: id.to_string(),
        nom: id.to_string(),
        famille: "Test".to_string(),
        hauteur_cm: 100,
        profondeur_racines_cm,
        taux_croissance,
        espacement_min_cm,
        exposition: Exposition::PleinSoleil,
        temperature_c: TemperatureC {
            min_survie: temp_min_survie,
            optimal_min: temp_min_survie + 5,
            optimal_max: temp_max_survie - 5,
            max_survie: temp_max_survie,
        },
        besoins_nutriments: BesoinsNutriments {
            n: Niveau::Moyen,
            p: Niveau::Faible,
            k: Niveau::Moyen,
        },
        besoin_eau: Niveau::Moyen,
        enracinement_eau,
        substrats_compatibles: substrats,
        saisons,
        rendement: Rendement {
            valeur: 1.0,
            unite: "kg/m2".to_string(),
            duree_jours: 90,
            source: None,
            niveau_confiance: Some("faible".to_string()),
        },
    }
}

/// Construit un bac de test (marge de bord fixée à 5 cm).
#[allow(clippy::too_many_arguments)]
fn bac_test(
    longueur: u32,
    largeur: u32,
    hauteur_terre_cm: u32,
    substrats: Vec<String>,
    saison_cible: Saison,
    temp_min: i32,
    temp_max: i32,
    drainage_eau: bool,
) -> Bac {
    Bac {
        id: "test_bac".to_string(),
        nom: "Bac Test".to_string(),
        dimensions_cm: DimensionsCm {
            longueur,
            largeur,
            hauteur: hauteur_terre_cm + 5,
        },
        hauteur_terre_cm,
        marge_bord_cm: 5,
        substrats,
        compost_disponible: true,
        drainage_eau,
        exposition: Exposition::PleinSoleil,
        saison_cible,
        temp_moyenne_c: TempMoyenneC { min: temp_min, max: temp_max },
        protection: Protection::Exterieur,
    }
}

/// Configuration minimale pour les tests (seuil invasif paramétrable).
fn config_test(seuil_invasif_cm2: i64) -> Config {
    Config {
        algorithme: ga_plantes::cli::Algorithme {
            population: 10,
            generations_max: 100,
            taux_croisement: 0.8,
            taux_mutation: 0.1,
            elitisme: 2,
            selection: "tournoi".to_string(),
            seed: 42,
            early_stopping: ga_plantes::cli::EarlyStopping {
                patience: 30,
                seuil_amelioration: 0.001,
            },
            taille_cellule_cm: 10,
            seuil_invasif_cm2,
        },
        chemins: ga_plantes::cli::Chemins {
            plantes: "data/plantes.json".to_string(),
            bacs: "data/bacs.json".to_string(),
            interactions: "data/interactions.json".to_string(),
        },
    }
}

fn s(v: &str) -> String {
    v.to_string()
}

// --- 1. Surface ---

#[test]
fn test_surface_valide() {
    // Surface utile : (100-10) x (100-10) = 8100 cm² ; plante : 50² = 2500 cm²
    let bac = bac_test(100, 100, 50, vec![s("terreau")], Saison::Ete, 15, 30, false);
    let plantes = vec![plante_test("tomate", 50, 40, false, vec![s("terreau")], vec![Saison::Ete], 10, 35, TauxCroissance::Moyenne)];
    let c = Combinaison::new(plantes, bac);
    assert!(est_valide(&c, &config_test(10000)).is_ok());
}

#[test]
fn test_surface_invalide() {
    // Surface utile : (50-10)² = 1600 cm² ; plantes : 2500 + 6400 = 8900 cm²
    let bac = bac_test(50, 50, 50, vec![s("terreau")], Saison::Ete, 15, 30, false);
    let plantes = vec![
        plante_test("tomate", 50, 40, false, vec![s("terreau")], vec![Saison::Ete], 10, 35, TauxCroissance::Moyenne),
        plante_test("courgette", 80, 50, false, vec![s("terreau")], vec![Saison::Ete], 12, 35, TauxCroissance::Invasive),
    ];
    let c = Combinaison::new(plantes, bac);
    assert!(matches!(
        est_valide(&c, &config_test(100000)),
        Err(ValidationError::SurfaceInsuffisante(_, _))
    ));
}

// --- 2. Racines ---

#[test]
fn test_racines_valide() {
    let bac = bac_test(100, 100, 50, vec![s("terreau")], Saison::Ete, 15, 30, false);
    let plantes = vec![plante_test("carotte", 15, 30, false, vec![s("terreau")], vec![Saison::Ete], 10, 25, TauxCroissance::Lente)];
    let c = Combinaison::new(plantes, bac);
    assert!(est_valide(&c, &config_test(10000)).is_ok());
}

#[test]
fn test_racines_invalide() {
    // Racines 30 cm > terre 20 cm
    let bac = bac_test(100, 100, 20, vec![s("terreau")], Saison::Ete, 15, 30, false);
    let plantes = vec![plante_test("carotte", 15, 30, false, vec![s("terreau")], vec![Saison::Ete], 10, 25, TauxCroissance::Lente)];
    let c = Combinaison::new(plantes, bac);
    assert!(matches!(
        est_valide(&c, &config_test(10000)),
        Err(ValidationError::RacinesTropProfondes(30, 20))
    ));
}

// --- 3. Enracinement eau ---

#[test]
fn test_hydro_valide() {
    // Bac hydroponique : laitue (enracinement_eau) acceptée.
    // Les contraintes terre/substrat sont neutralisées (hauteur_terre 0, substrats vides).
    let bac = bac_test(100, 100, 0, vec![], Saison::Ete, 15, 30, true);
    let plantes = vec![plante_test("laitue", 25, 15, true, vec![s("fibre_de_coco")], vec![Saison::Ete], 10, 25, TauxCroissance::Moyenne)];
    let c = Combinaison::new(plantes, bac);
    assert!(est_valide(&c, &config_test(10000)).is_ok());
}

#[test]
fn test_hydro_invalide_plante_terrestre() {
    // Plante aquatique dans un bac classique -> invalide
    let bac = bac_test(100, 100, 30, vec![s("terreau")], Saison::Ete, 15, 30, false);
    let plantes = vec![plante_test("laitue", 25, 15, true, vec![s("terreau")], vec![Saison::Ete], 10, 35, TauxCroissance::Moyenne)];
    let c = Combinaison::new(plantes, bac);
    assert!(matches!(
        est_valide(&c, &config_test(10000)),
        Err(ValidationError::EnracinementEauIncompatible(id)) if id == "laitue"
    ));
}

// --- 4. Substrat ---

#[test]
fn test_substrat_invalide() {
    // La plante veut du sableux, le bac n'offre que du terreau
    let bac = bac_test(100, 100, 30, vec![s("terreau")], Saison::Ete, 15, 30, false);
    let plantes = vec![plante_test("carotte", 15, 30, false, vec![s("sableux")], vec![Saison::Ete], 10, 25, TauxCroissance::Lente)];
    let c = Combinaison::new(plantes, bac);
    assert!(matches!(
        est_valide(&c, &config_test(10000)),
        Err(ValidationError::SubstratIncompatible(_, _, _))
    ));
}

// --- 5. Saison ---

#[test]
fn test_saison_invalide() {
    // La plante est de printemps, le bac cible l'été
    let bac = bac_test(100, 100, 30, vec![s("terreau")], Saison::Ete, 15, 30, false);
    let plantes = vec![plante_test("carotte", 15, 30, false, vec![s("terreau")], vec![Saison::Printemps], 10, 25, TauxCroissance::Lente)];
    let c = Combinaison::new(plantes, bac);
    assert!(matches!(
        est_valide(&c, &config_test(10000)),
        Err(ValidationError::SaisonIncompatible(_, _, _))
    ));
}

// --- 6. Température ---

#[test]
fn test_temperature_invalide() {
    // Plante : survie [10-20]°C ; bac : [25-30]°C -> plages disjointes
    let bac = bac_test(100, 100, 30, vec![s("terreau")], Saison::Ete, 25, 30, false);
    let plantes = vec![plante_test("laitue", 25, 15, false, vec![s("terreau")], vec![Saison::Ete], 10, 20, TauxCroissance::Moyenne)];
    let c = Combinaison::new(plantes, bac);
    assert!(matches!(
        est_valide(&c, &config_test(10000)),
        Err(ValidationError::TemperatureIncompatible(_, 10, 20, 25, 30))
    ));
}

// --- 7. Invasive ---

#[test]
fn test_invasive_valide() {
    // Surface utile : 190² = 36100 cm² >= seuil 30000 -> OK
    let bac = bac_test(200, 200, 50, vec![s("terreau")], Saison::Ete, 15, 30, false);
    let plantes = vec![plante_test("courgette", 80, 50, false, vec![s("terreau")], vec![Saison::Ete], 12, 35, TauxCroissance::Invasive)];
    let c = Combinaison::new(plantes, bac);
    assert!(est_valide(&c, &config_test(30000)).is_ok());
}

#[test]
fn test_invasive_invalide() {
    // Surface utile : 90² = 8100 cm² < seuil 10000 -> refusée
    let bac = bac_test(100, 100, 50, vec![s("terreau")], Saison::Ete, 15, 30, false);
    let plantes = vec![plante_test("courgette", 80, 50, false, vec![s("terreau")], vec![Saison::Ete], 12, 35, TauxCroissance::Invasive)];
    let c = Combinaison::new(plantes, bac);
    assert!(matches!(
        est_valide(&c, &config_test(10000)),
        Err(ValidationError::InvasiveInterdite(id, 8100, 10000)) if id == "courgette"
    ));
}

// --- Cas limites ---

#[test]
fn test_bac_vide_autorise() {
    let bac = bac_test(100, 100, 30, vec![s("terreau")], Saison::Ete, 15, 30, false);
    let c = Combinaison::new(vec![], bac);
    assert!(est_valide(&c, &config_test(10000)).is_ok());
}
