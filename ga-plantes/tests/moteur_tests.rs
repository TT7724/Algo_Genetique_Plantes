use ga_plantes::cli::{load_and_validate_data, Config};
use ga_plantes::ga::evolution::{evoluer, EvolutionError};
use ga_plantes::models::{
    Bac, BesoinsNutriments, Combinaison, DimensionsCm, Exposition, Niveau, Plante, Protection,
    Rendement, Saison, TauxCroissance, TempMoyenneC, TemperatureC,
};

// --- Fixtures ---

fn plante_test(id: &str, espacement_min_cm: u32, hauteur_cm: u32) -> Plante {
    Plante {
        id: id.to_string(),
        nom: id.to_string(),
        famille: "Test".to_string(),
        hauteur_cm,
        profondeur_racines_cm: 10,
        taux_croissance: TauxCroissance::Moyenne,
        espacement_min_cm,
        exposition: Exposition::PleinSoleil,
        temperature_c: TemperatureC {
            min_survie: 10,
            optimal_min: 15,
            optimal_max: 25,
            max_survie: 30,
        },
        besoins_nutriments: BesoinsNutriments {
            n: Niveau::Moyen,
            p: Niveau::Faible,
            k: Niveau::Moyen,
        },
        besoin_eau: Niveau::Moyen,
        enracinement_eau: false,
        substrats_compatibles: vec!["terreau".to_string()],
        saisons: vec![Saison::Ete],
        rendement: Rendement {
            valeur: 1.0,
            unite: "kg/m2".to_string(),
            duree_jours: 90,
            source: None,
            niveau_confiance: Some("faible".to_string()),
        },
    }
}

/// Bac spacieux (surface utile 190x190 = 36100 cm²), 30 cm de terre, été.
fn bac_test() -> Bac {
    Bac {
        id: "test_bac".to_string(),
        nom: "Bac Test".to_string(),
        dimensions_cm: DimensionsCm {
            longueur: 200,
            largeur: 200,
            hauteur: 35,
        },
        hauteur_terre_cm: 30,
        marge_bord_cm: 5,
        substrats: vec!["terreau".to_string()],
        compost_disponible: true,
        drainage_eau: false,
        exposition: Exposition::PleinSoleil,
        saison_cible: Saison::Ete,
        temp_moyenne_c: TempMoyenneC { min: 15, max: 30 },
        protection: Protection::Exterieur,
    }
}

fn config_test(seed: u64, generations_max: usize, patience: usize, taux_mutation: f64, taux_croisement: f64) -> Config {
    Config {
        algorithme: ga_plantes::cli::Algorithme {
            population: 10,
            generations_max,
            taux_croisement,
            taux_mutation,
            elitisme: 2,
            selection: "tournoi".to_string(),
            seed,
            early_stopping: ga_plantes::cli::EarlyStopping {
                patience,
                seuil_amelioration: 0.001,
            },
            taille_cellule_cm: 10,
            seuil_invasif_cm2: 10000,
        },
        chemins: ga_plantes::cli::Chemins {
            plantes: "data/plantes.json".to_string(),
            bacs: "data/bacs.json".to_string(),
            interactions: "data/interactions.json".to_string(),
        },
    }
}

/// Score factice déterministe : somme des hauteurs des plantes.
fn score_simple(combinaison: &Combinaison) -> f64 {
    combinaison.plantes.iter().map(|p| p.hauteur_cm as f64).sum()
}

// --- Tests ---

#[test]
fn test_reproductibilite_seed() {
    let pool = vec![plante_test("tomate", 50, 100), plante_test("laitue", 25, 50)];
    let bac = bac_test();
    let population_initiale = vec![
        Combinaison::new(vec![plante_test("tomate", 50, 100)], bac.clone()),
        Combinaison::new(vec![plante_test("laitue", 25, 50)], bac.clone()),
    ];
    let config = config_test(42, 10, 5, 0.1, 0.8);

    // Deux exécutions avec la même seed doivent être identiques
    let r1 = evoluer(population_initiale.clone(), &pool, &config, 2, score_simple).unwrap();
    let r2 = evoluer(population_initiale, &pool, &config, 2, score_simple).unwrap();

    assert_eq!(r1.generations, r2.generations);
    assert_eq!(format!("{:?}", r1.meilleur_individu), format!("{:?}", r2.meilleur_individu));
}

#[test]
fn test_elitisme_conserve_le_meilleur() {
    let pool = vec![
        plante_test("tomate", 50, 100),
        plante_test("laitue", 25, 50),
        plante_test("courgette", 80, 200),
    ];
    let bac = bac_test();
    let population_initiale = vec![
        Combinaison::new(vec![plante_test("tomate", 50, 100)], bac.clone()),
        Combinaison::new(vec![plante_test("laitue", 25, 50)], bac.clone()),
        Combinaison::new(vec![plante_test("courgette", 80, 200)], bac.clone()),
    ];
    let config = config_test(42, 100, 30, 0.1, 0.8);

    let result = evoluer(population_initiale, &pool, &config, 2, score_simple).unwrap();
    // L'élitisme garantit que le meilleur jamais vu n'est jamais perdu
    assert!(score_simple(&result.meilleur_individu) >= 200.0);
}

#[test]
fn test_early_stopping() {
    let pool = vec![plante_test("tomate", 50, 50)];
    let bac = bac_test();
    let population_initiale = vec![
        Combinaison::new(vec![plante_test("tomate", 50, 50)], bac.clone()),
        Combinaison::new(vec![plante_test("tomate", 50, 50)], bac.clone()),
    ];
    // Mutation et croisement désactivés : la population ne change jamais,
    // le score stagne -> arrêt après `patience` générations, bien avant 100.
    let config = config_test(42, 100, 5, 0.0, 0.0);

    let result = evoluer(population_initiale, &pool, &config, 2, score_simple).unwrap();
    assert!(result.generations < 100, "Early stopping non déclenché : {} générations", result.generations);
}

#[test]
fn test_erreur_population_vide() {
    let pool = vec![plante_test("tomate", 50, 50)];
    let config = config_test(42, 10, 5, 0.1, 0.8);

    let result = evoluer(vec![], &pool, &config, 2, score_simple);
    assert!(matches!(result, Err(EvolutionError::PopulationVide)));
}

#[test]
fn test_erreur_taille_tournoi_invalide() {
    let pool = vec![plante_test("tomate", 50, 50)];
    let bac = bac_test();
    let population_initiale = vec![Combinaison::new(vec![], bac)];
    let config = config_test(42, 10, 5, 0.1, 0.8);

    // taille_tournoi = 0 : invalide
    let result = evoluer(population_initiale, &pool, &config, 0, score_simple);
    assert!(result.is_err());
}
