use axum::http::HeaderMap;
use axum::http::header::COOKIE;

pub const NOMBRE_COOKIE_SESION: &str = "alumframe_sesion";
const DURACION_EN_SEGUNDOS: u32 = 12 * 60 * 60;

pub fn leer_cookie_de_sesion(cabeceras: &HeaderMap) -> Option<String> {
    cabeceras
        .get_all(COOKIE)
        .iter()
        .filter_map(|valor| valor.to_str().ok())
        .flat_map(|linea| linea.split(';'))
        .filter_map(|par| par.trim().split_once('='))
        .find(|(nombre, _)| *nombre == NOMBRE_COOKIE_SESION)
        .map(|(_, valor)| valor.to_owned())
        .filter(|valor| !valor.is_empty())
}

pub fn cookie_de_sesion(token: &str) -> String {
    format!(
        "{NOMBRE_COOKIE_SESION}={token}; HttpOnly; SameSite=Lax; Path=/; Max-Age={DURACION_EN_SEGUNDOS}"
    )
}

pub fn cookie_de_cierre() -> String {
    format!("{NOMBRE_COOKIE_SESION}=; HttpOnly; SameSite=Lax; Path=/; Max-Age=0")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encuentra_la_cookie_entre_otras() {
        let mut cabeceras = HeaderMap::new();
        cabeceras.insert(
            COOKIE,
            "tema=claro; alumframe_sesion=abc123; otra=1"
                .parse()
                .unwrap(),
        );
        assert_eq!(leer_cookie_de_sesion(&cabeceras), Some("abc123".into()));
    }

    #[test]
    fn ignora_la_cookie_vacia_o_ausente() {
        let mut cabeceras = HeaderMap::new();
        assert_eq!(leer_cookie_de_sesion(&cabeceras), None);
        cabeceras.insert(COOKIE, "alumframe_sesion=".parse().unwrap());
        assert_eq!(leer_cookie_de_sesion(&cabeceras), None);
    }

    #[test]
    fn la_cookie_es_http_only_y_same_site() {
        let cookie = cookie_de_sesion("t");
        assert!(cookie.contains("HttpOnly"));
        assert!(cookie.contains("SameSite=Lax"));
    }
}
