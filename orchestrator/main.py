"""Orchestrateur GA Potager : charge les données, appelle la lib Rust
(CLI subprocess), lit les résultats JSON et génère les graphiques.
TODO (M3) : implémenter.
"""
import argparse


def main() -> None:
    parser = argparse.ArgumentParser(description="Orchestrateur GA Potager")
    parser.add_argument("--config", default="data/config.json")
    args = parser.parse_args()
    # TODO (M3-1) : appeler ga-plantes via subprocess
    # TODO (M3-2) : graphiques matplotlib (fitness min/moy/max par génération)
    print(f"Config : {args.config}")


if __name__ == "__main__":
    main()
