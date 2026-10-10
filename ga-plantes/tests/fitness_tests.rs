//! Tests de la fitness factice. Les tests de la vraie fitness
//! seront écrits par l'utilisateur.
use ga_plantes::fitness::fitness_stub;

#[test]
fn fitness_stub_est_croissante() {
    assert!(fitness_stub(&["a".into()]) <= fitness_stub(&["a".into(), "b".into()]));
}

#[test]
fn fitness_stub_vide() {
    assert_eq!(fitness_stub(&[]), 0.0);
}
