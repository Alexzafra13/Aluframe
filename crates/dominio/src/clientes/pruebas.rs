use std::collections::HashSet;

use proptest::prelude::*;

use super::*;

#[test]
fn un_cliente_en_blanco_es_valido() {
    let datos = validar_cliente(&EntradaDeCliente::default()).unwrap();
    assert_eq!(datos, DatosDeCliente::default());
    assert_eq!(datos.tipo, TipoCliente::Particular);
    assert_eq!(
        nombre_visible(12, datos.nombre.as_deref()),
        "Cliente sin nombre (n.º 12)"
    );
}

#[test]
fn limpia_y_normaliza_lo_escrito() {
    let entrada = EntradaDeCliente {
        tipo: "Empresa".into(),
        nombre: "  Construcciones Peña SL ".into(),
        nif: "b-12345674".into(),
        telefono: "+34 924 55 55 55".into(),
        fiscal_localidad: "Zafra".into(),
        ..EntradaDeCliente::default()
    };
    let datos = validar_cliente(&entrada).unwrap();
    assert_eq!(datos.tipo, TipoCliente::Empresa);
    assert_eq!(datos.nombre.as_deref(), Some("Construcciones Peña SL"));
    assert_eq!(datos.nif.unwrap().como_texto(), "B12345674");
    assert_eq!(nombre_visible(3, Some("  ")), "Cliente sin nombre (n.º 3)");
}

#[test]
fn la_busqueda_ignora_tildes() {
    let entrada = EntradaDeCliente {
        nombre: "José Peña".into(),
        fiscal_localidad: "Zafra".into(),
        ..EntradaDeCliente::default()
    };
    let datos = validar_cliente(&entrada).unwrap();
    assert_eq!(datos.texto_de_busqueda(), "jose pena zafra");
}

#[test]
fn senala_cada_campo_mal_escrito() {
    let entrada = EntradaDeCliente {
        tipo: "autonomo".into(),
        nif: "12345678A".into(),
        email: "sin-arroba".into(),
        telefono: "llámame".into(),
        nombre: "x".repeat(201),
        ..EntradaDeCliente::default()
    };
    let errores = validar_cliente(&entrada).unwrap_err();
    let campos: Vec<&str> = errores.iter().map(|e| e.campo).collect();
    assert_eq!(campos, vec!["tipo", "nombre", "nif", "email", "telefono"]);
}

#[test]
fn comprueba_correo_y_telefono() {
    let con = |email: &str, telefono: &str| {
        validar_cliente(&EntradaDeCliente {
            email: email.into(),
            telefono: telefono.into(),
            ..EntradaDeCliente::default()
        })
    };
    assert!(con("ana@correo.es", "924 000 000").is_ok());
    assert!(con("ana@correo", "").is_err());
    assert!(con("a b@correo.es", "").is_err());
    assert!(con("", "12345").is_err());
}

#[test]
fn una_direccion_en_blanco_no_se_guarda() {
    assert_eq!(validar_direccion(&EntradaDeDireccion::default()), Ok(None));
    let entrada = EntradaDeDireccion {
        etiqueta: "Obra".into(),
        calle: "Calle Mayor 3".into(),
        localidad: "Zafra".into(),
        ..EntradaDeDireccion::default()
    };
    let datos = validar_direccion(&entrada).unwrap().unwrap();
    assert_eq!(datos.resumen(), "Obra · Calle Mayor 3, Zafra");
    let solo_etiqueta = DatosDeDireccion {
        etiqueta: Some("Casa".into()),
        ..DatosDeDireccion::default()
    };
    assert_eq!(solo_etiqueta.resumen(), "Casa");
    assert_eq!(DatosDeDireccion::default().resumen(), "Dirección sin datos");
    let mal = EntradaDeDireccion {
        contacto_telefono: "no".into(),
        ..EntradaDeDireccion::default()
    };
    assert_eq!(
        validar_direccion(&mal).unwrap_err()[0].campo,
        "contacto_telefono"
    );
}

#[test]
fn importa_clientes_y_senala_los_errores_por_linea() {
    let texto = "Nombre;NIF;Teléfono;Localidad;Extra\nAna;12345678Z;924000000;Zafra;x\n;;;;\nLuis;12345678A;;;\nEva;12.345.678-z;;;\n";
    let (filas, informe) = leer_clientes(texto);
    assert_eq!(filas.len(), 1);
    assert_eq!(filas[0].linea, 2);
    assert_eq!(filas[0].datos.fiscal_localidad.as_deref(), Some("Zafra"));
    let errores: Vec<String> = informe.errores.iter().map(ToString::to_string).collect();
    assert!(errores[0].starts_with("línea 4: nif:"), "{errores:?}");
    assert_eq!(errores[1], "línea 5: el NIF ya aparece en la línea 2");
    assert!(informe.avisos[0].mensaje.contains("«extra» no se usa"));
}

#[test]
fn exige_alguna_columna_conocida_y_alguna_fila() {
    assert!(leer_clientes("cosa;otra\n1;2\n").1.tiene_errores());
    assert!(leer_clientes("nombre\n").1.tiene_errores());
    assert!(leer_clientes("").1.tiene_errores());
}

#[test]
fn salta_los_nif_que_ya_existen() {
    let (filas, mut informe) = leer_clientes("nombre;nif\nAna;12345678Z\nSin NIF;\n");
    let existentes = HashSet::from(["12345678Z".to_owned()]);
    let quedan = apartar_existentes(filas, &existentes, &mut informe);
    assert_eq!(quedan.len(), 1);
    assert_eq!(quedan[0].datos.nombre.as_deref(), Some("Sin NIF"));
    assert!(
        informe.avisos[0]
            .to_string()
            .contains("línea 2: ya hay un cliente con el NIF 12345678Z")
    );
}

proptest! {
    #[test]
    fn cualquier_texto_da_clientes_o_errores_sin_romperse(texto in "\\PC{0,300}") {
        let _ = leer_clientes(&format!("nombre;nif;email\n{texto}"));
    }
}
