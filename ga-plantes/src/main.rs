use anyhow::{bail, Context, Result};
use clap::Parser;
use ga_plantes::cli::{load_and_validate_data, Args};
use ga_plantes::fitness::Evaluateur;
use ga_plantes::ga::evolution::evoluer;
use ga_plantes::ga::validation::est_valide;
use ga_plantes::models::{Combinaison, Plante};
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;

fn main() -> Result<()> {
    // 1. Parse les arguments CLI
    let args = Args::parse();

    // 2. Charge et valide les données (propage l'erreur avec anyhow)
    let (plantes, bacs, interactions, mut config) =
        load_and_validate_data(&args.data_dir, &args.config_path)
            .context("Échec du chargement ou de la validation des données")?;

    // 3. Surcharge la seed si fournie via CLI
    if let Some(seed) = args.seed {
        config.algorithme.seed = seed;
    }

    // 4. Construit l'évaluateur de fitness
    let evaluateur = Evaluateur::new(interactions);

    // 5. Génère la population initiale par construction GLOUTONNE :
    //    partir d'un bac vide et ajouter des plantes une à une (ordre
    //    aléatoire) tant que la combinaison reste valide. Contrairement au
    //    tirage aléatoire "1 à 5 plantes", cette stratégie trouve des
    //    combinaisons valides même avec des contraintes strictes.
    let mut rng = StdRng::seed_from_u64(config.algorithme.seed);
    let mut population: Vec<Combinaison> = Vec::new();
    let mut tentatives = 0usize;
    const MAX_TENTATIVES: usize = 10_000;

    while population.len() < config.algorithme.population {
        tentatives += 1;
        if tentatives > MAX_TENTATIVES {
            bail!(
                "Impossible de générer {} combinaisons valides en {} tentatives — \\
                 aucune plante n'est peut-être compatible avec les bacs (saison, \\
                 substrat, profondeur de terre, drainage)",
                config.algorithme.population,
                MAX_TENTATIVES
            );
        }

        if let Some(combinaison) = construire_individu(&plantes, &bacs, &config, &mut rng) {
            population.push(combinaison);
        }
    }

    // 6. Exécute l'évolution
    let result = evoluer(
        population,
        &plantes,
        &config,
        3, // taille_tournoi = 3 (en attendant un champ dédié dans config.json)
        |c| evaluateur.evaluer(c),
    )
    .context("Échec de l'évolution")?;

    // 7. Affiche le potager gagnant
    println!("=== Potager gagnant ===");
    println!(
        "Bac : {} ({} cm x {} cm)",
        result.meilleur_individu.bac.nom,
        result.meilleur_individu.bac.dimensions_cm.longueur,
        result.meilleur_individu.bac.dimensions_cm.largeur
    );
    println!("Générations : {}", result.generations);
    println!("Plantes :");
    for plante in &result.meilleur_individu.plantes {
        println!(
            "  - {} (famille: {}, hauteur: {} cm)",
            plante.nom, plante.famille, plante.hauteur_cm
        );
    }
    println!(
        "Score final : {:.2}",
        evaluateur.evaluer(&result.meilleur_individu)
    );

    Ok(())
}

/// Construit un individu valide par ajout glouton :
/// un bac au hasard, puis les plantes (ordre mélangé) une à une, gardées
/// seulement si la combinaison reste valide. Retourne None si aucune
/// plante ne convient (le binaire réessaie / abandonne au plafond).
fn construire_individu(
    plantes: &[Plante],
    bacs: &[ga_plantes::models::Bac],
    config: &ga_plantes::cli::Config,
    rng: &mut StdRng,
) -> Option<Combinaison> {
    let bac = bacs.choose(rng).expect("bacs.json vide").clone();

    let mut candidates: Vec<Plante> = plantes.to_vec();
    candidates.shuffle(rng);

    let mut choisies: Vec<Plante> = Vec::new();
    for plante in candidates {
        if choisies.len() >= 5 {
            break; // taille maximale d'une combinaison initiale
        }
        let mut essai = choisies.clone();
        essai.push(plante.clone());
        if est_valide(&Combinaison::new(essai, bac.clone()), config).is_ok() {
            choisies.push(plante);
        }
    }

    if choisies.is_empty() {
        return None; // aucune plante compatible avec ce bac
    }
    Some(Combinaison::new(choisies, bac))
}
