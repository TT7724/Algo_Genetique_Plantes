use anyhow::{bail, Context, Result};
use clap::Parser;
use ga_plantes::cli::{load_and_validate_data, Args};
use ga_plantes::fitness::Evaluateur;
use ga_plantes::ga::evolution::evoluer;
use ga_plantes::ga::validation::est_valide;
use ga_plantes::models::Combinaison;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};

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

    // 5. Génère la population initiale (taille = config.algorithme.population)
    //    Garde anti-boucle-infinie : si les données ne permettent aucune
    //    combinaison valide, on abandonne après un plafond de tentatives.
    let mut rng = StdRng::seed_from_u64(config.algorithme.seed);
    let mut population: Vec<Combinaison> = Vec::new();
    let mut tentatives = 0usize;
    const MAX_TENTATIVES: usize = 10_000;

    while population.len() < config.algorithme.population {
        tentatives += 1;
        if tentatives > MAX_TENTATIVES {
            bail!(
                "Impossible de générer {} combinaisons valides en {} tentatives — les contraintes des données sont peut-être trop strictes",
                config.algorithme.population,
                MAX_TENTATIVES
            );
        }

        // Choisit un bac aléatoire et un sous-ensemble aléatoire de plantes
        let bac = bacs.choose(&mut rng).expect("bacs.json vide").clone();
        let subset_size = rng.gen_range(1..=5.min(plantes.len()));
        let subset: Vec<_> = plantes
            .choose_multiple(&mut rng, subset_size)
            .cloned()
            .collect();

        // Ne garde que les combinaisons valides
        let combinaison = Combinaison::new(subset, bac);
        if est_valide(&combinaison, &config).is_ok() {
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

    // 7a. Mode JSON : sortie structurée pour le notebook (configuré en amont,
    //     il écrit lui-même bacs.json puis parse ce résultat)
    if args.json {
        let detail = evaluateur.evaluer_detail(&result.meilleur_individu);
        let sortie = serde_json::json!({
            "seed": config.algorithme.seed,
            "generations": result.generations,
            "meilleur": {
                "bac": result.meilleur_individu.bac.id,
                "plantes": result.meilleur_individu.plantes.iter().map(|p| serde_json::json!({
                    "id": p.id, "nom": p.nom, "famille": p.famille,
                })).collect::<Vec<_>>(),
                "score": detail.total,
                "detail": {
                    "rendement": detail.rendement * 5.0,
                    "interactions": detail.interactions * 7.0,
                    "occupation": detail.occupation * 10.0,
                    "interactions_actives": detail.interactions_actives,
                },
            },
            "population": result.population_finale.iter().map(|c| serde_json::json!({
                "plantes": c.plantes.iter().map(|p| p.id.clone()).collect::<Vec<_>>(),
                "score": evaluateur.evaluer(c),
            })).collect::<Vec<_>>(),
        });
        println!("{}", serde_json::to_string_pretty(&sortie)?);
        return Ok(());
    }

    // 7b. Mode texte : le potager gagnant, lisible dans un terminal
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
