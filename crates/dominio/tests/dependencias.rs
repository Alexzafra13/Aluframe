const MANIFIESTO: &str = include_str!("../Cargo.toml");
const PERMITIDAS: [&str; 2] = ["serde", "thiserror"];

#[test]
fn el_dominio_solo_depende_de_serde_y_thiserror() {
    let seccion = MANIFIESTO
        .split("[dependencies]")
        .nth(1)
        .and_then(|resto| resto.split("\n[").next())
        .unwrap_or_default();
    let nombres: Vec<&str> = seccion
        .lines()
        .filter_map(|linea| linea.split('=').next())
        .map(str::trim)
        .filter(|nombre| !nombre.is_empty() && !nombre.starts_with('#'))
        .collect();
    assert!(!nombres.is_empty());
    for nombre in nombres {
        assert!(PERMITIDAS.contains(&nombre), "dependencia no permitida: {nombre}");
    }
}
