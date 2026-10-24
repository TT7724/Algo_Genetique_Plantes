use ga_plantes::cli::{Confiance, Intensite, Interaction, InteractionType};
use ga_plantes::fitness::Evaluateur;
use ga_plantes::models::{
    Bac, BesoinsNutriments, Combinaison, DimensionsCm, Exposition, Niveau, Plante, Protection,
    Rendement, Saison, TauxCroissance, TempMoyenneC, TemperatureC,
};

// --- Fixtures ---

fn plante_test(id: &str, espacement_min_cm: u32, rendement: f32) -> Plante {
    Plante {
        id: id.to_string(),
        nom: id.to_string(),
        famille: "Test".to_string(),
        hauteur_cm: 50,
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
            valeur: rendement,
            unite: "kg/m2".to_string(),
            duree_jours: 90,
            source: None,
            niveau_confiance: Some("faible".to_string()),
        },
    }
}

fn bac_test() -> Bac {
    Bac {
        id: "test_bac".to_string(),
        nom: "Bac Test".to_string(),
        dimensions_cm: DimensionsCm { longueur: 100, largeur: 100, hauteur: 35 },
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

fn interaction_test(a: &str, b: &str, r#type: InteractionType, intensite: Intensite) -> Interaction {
    Interaction {
        plante_a: a.to_string(),
        plante_b: b.to_string(),
        r#type,
        intensite,
        confiance: Confiance::Moyenne,
    }
}

// --- Tests ---

#[test]
fn test_bac_vide_score_zero() {
    let evaluateur = Evaluateur::new(vec![]);
    let c = Combinaison::new(vec![], bac_test());
    assert_eq!(evaluateur.evaluer(&c), 0.0);
}

#[test]
fn test_compagnonnage_ameliore_le_score() {
    // Deux plantes identiques, seule l'interaction change
    let plantes = vec![plante_test("tomate", 50, 3.0), plante_test("basilic", 50, 1.0)];
    let bac = bac_test();

    let compagnonne = Evaluateur::new(vec![interaction_test(
        "tomate", "basilic", InteractionType::Compagnonnage, Intensite::Forte,
    )]);
    let neutre = Evaluateur::new(vec![]);

    let c = Combinaison::new(plantes, bac);
    assert!(compagnonne.evaluer(&c) > neutre.evaluer(&c));
}

#[test]
fn test_competition_penalise_le_score() {
    let plantes = vec![plante_test("tomate", 50, 3.0), plante_test("courgette", 80, 4.0)];
    let bac = bac_test();

    let concurrente = Evaluateur::new(vec![interaction_test(
        "tomate", "courgette", InteractionType::CompetitionEau, Intensite::Forte,
    )]);
    let neutre = Evaluateur::new(vec![]);

    let c = Combinaison::new(plantes, bac);
    assert!(concurrente.evaluer(&c) < neutre.evaluer(&c));
}

#[test]
fn test_interaction_absente_ignoree() {
    // Une interaction entre deux plantes non présentes ne compte pas
    let evaluateur = Evaluateur::new(vec![interaction_test(
        "tomate", "basilic", InteractionType::Compagnonnage, Intensite::Forte,
    )]);
    let c = Combinaison::new(vec![plante_test("carotte", 15, 2.0)], bac_test());
    let attendu = 2.0 * 5.0 + ((15.0 * 15.0) / (90.0 * 90.0)) * 10.0;
    assert!((evaluateur.evaluer(&c) - attendu).abs() < 1e-9);
}

#[test]
fn test_occupation_recompense_le_remplissage() {
    let evaluateur = Evaluateur::new(vec![]);
    let bac = bac_test();
    // Une seule plante vs deux : le score doit augmenter (rendement + occupation)
    let une = Combinaison::new(vec![plante_test("tomate", 30, 1.0)], bac.clone());
    let deux = Combinaison::new(vec![plante_test("tomate", 30, 1.0), plante_test("basilic", 20, 1.0)], bac);
    assert!(evaluateur.evaluer(&deux) > evaluateur.evaluer(&une));
}
