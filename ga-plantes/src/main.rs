//! Point d'entrée CLI : `cargo run -- --config data/config.json`

use clap::Parser;

/// Trouve les meilleures combinaisons de plantes pour un bac.
#[derive(Parser)]
struct Args {
    /// Chemin vers le fichier de configuration JSON
    #[arg(long)]
    config: String,
}

fn main() {
    let args = Args::parse();
    // TODO (M2) : charger la config, exécuter l'AG, écrire le JSON de sortie.
    println!("Config chargée depuis : {}", args.config);
}
