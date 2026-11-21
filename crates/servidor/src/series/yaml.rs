use dominio::series::Valor;
use yaml_rust2::{Yaml, YamlLoader};

/// Convierte un archivo YAML en el árbol neutro del dominio; los números se pasan como texto.
pub fn leer_yaml(texto: &str) -> Result<Valor, String> {
    let documentos = YamlLoader::load_from_str(texto)
        .map_err(|error| format!("El archivo no es YAML válido: {error}"))?;
    match documentos.as_slice() {
        [documento] => convertir(documento),
        [] => Err("El archivo está vacío.".into()),
        _ => Err("El archivo tiene más de un documento YAML (---).".into()),
    }
}

fn convertir(yaml: &Yaml) -> Result<Valor, String> {
    Ok(match yaml {
        Yaml::Real(texto) => Valor::Numero(texto.clone()),
        Yaml::Integer(entero) => Valor::Numero(entero.to_string()),
        Yaml::String(texto) => Valor::Texto(texto.clone()),
        Yaml::Boolean(valor) => Valor::Booleano(*valor),
        Yaml::Null => Valor::Nulo,
        Yaml::Array(elementos) => {
            Valor::Lista(elementos.iter().map(convertir).collect::<Result<_, _>>()?)
        }
        Yaml::Hash(mapa) => {
            let mut campos = Vec::with_capacity(mapa.len());
            for (clave, valor) in mapa {
                campos.push((clave_como_texto(clave)?, convertir(valor)?));
            }
            Valor::Mapa(campos)
        }
        Yaml::Alias(_) | Yaml::BadValue => {
            return Err("El archivo tiene una construcción YAML no admitida.".into());
        }
    })
}

fn clave_como_texto(clave: &Yaml) -> Result<String, String> {
    match clave {
        Yaml::String(texto) => Ok(texto.clone()),
        Yaml::Integer(entero) => Ok(entero.to_string()),
        _ => Err("Hay una clave que no es un texto.".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conserva_los_decimales_como_texto_y_el_orden() {
        let valor = leer_yaml("b: 4.85\na: 640\nc: CONFIRMAR\nd: [1, x]").unwrap();
        assert_eq!(
            valor,
            Valor::Mapa(vec![
                ("b".into(), Valor::Numero("4.85".into())),
                ("a".into(), Valor::Numero("640".into())),
                ("c".into(), Valor::Texto("CONFIRMAR".into())),
                (
                    "d".into(),
                    Valor::Lista(vec![Valor::Numero("1".into()), Valor::Texto("x".into())])
                ),
            ])
        );
    }

    #[test]
    fn rechaza_yaml_mal_formado_vacio_o_con_varios_documentos() {
        assert!(leer_yaml("a: [1, 2").is_err());
        assert!(leer_yaml("").is_err());
        assert!(leer_yaml("a: 1\n---\nb: 2").is_err());
    }

    #[test]
    fn resuelve_los_alias_al_valor_original() {
        let valor = leer_yaml("a: &x 1.5\nb: *x").unwrap();
        let Valor::Mapa(campos) = valor else {
            panic!("se esperaba un mapa")
        };
        assert_eq!(campos[1].1, Valor::Numero("1.5".into()));
    }
}
