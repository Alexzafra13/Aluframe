//! La 0005 pasa los cristales de los trabajos en marcha a la lista de material a pedir. Las filas
//! se crean sin el resto del presupuesto: con las claves ajenas apagadas solo para la prueba.

use super::*;

const AHORA: &str = "2026-11-25T10:00:00Z";

fn congelado(vidrios: &str) -> String {
    format!(r#"{{"resumen":{{"vidrios":{vidrios}}}}}"#)
}

fn con_trabajo(estado: &str, ventanas: &[(i64, i64, String)]) -> Connection {
    let mut conexion = Connection::open_in_memory().unwrap();
    aplicar_migraciones(&mut conexion, &MIGRACIONES[..4], AHORA).unwrap();
    conexion.pragma_update(None, "foreign_keys", "OFF").unwrap();
    conexion
        .execute(
            "INSERT INTO trabajo (id, anio, numero, presupuesto_id, cliente_id, cliente_nombre, \
             estado, creado_en) VALUES (1, 2026, 1, 1, 1, 'Cliente', ?1, ?2)",
            [estado, AHORA],
        )
        .unwrap();
    for (id, recibido, json) in ventanas {
        conexion
            .execute(
                "INSERT INTO ventana_trabajo (id, trabajo_id, orden, linea_orden, descripcion, \
                 cristal_recibido, congelado_json) VALUES (?1, 1, ?1, 1, 'Ventana', ?2, ?3)",
                rusqlite::params![id, recibido, json],
            )
            .unwrap();
    }
    preparar_esquema(&mut conexion, AHORA).unwrap();
    conexion
}

fn materiales(conexion: &Connection) -> Vec<(i64, i64, String, i64, i64, String)> {
    let mut consulta = conexion
        .prepare(
            "SELECT ventana_id, orden, codigo, cantidad, ancho_dmm, estado \
             FROM material_a_pedir ORDER BY ventana_id, orden",
        )
        .unwrap();
    consulta
        .query_map([], |f| {
            Ok((
                f.get(0)?,
                f.get(1)?,
                f.get(2)?,
                f.get(3)?,
                f.get(4)?,
                f.get(5)?,
            ))
        })
        .unwrap()
        .map(Result::unwrap)
        .collect()
}

const DOS_CRISTALES: &str =
    r#"[{"tipo":"CRISTAL","cantidad":2,"ancho_dmm":5020,"alto_dmm":10690}]"#;

#[test]
fn pasa_los_cristales_de_los_trabajos_en_marcha_y_respeta_lo_recibido() {
    let conexion = con_trabajo(
        "en_curso",
        &[
            (1, 0, congelado(DOS_CRISTALES)),
            (2, 1, congelado(DOS_CRISTALES)),
            (3, 0, congelado("[]")),
            (4, 0, "no es json".into()),
        ],
    );
    assert_eq!(
        materiales(&conexion),
        vec![
            (1, 1, "CRISTAL".into(), 2, 5020, "pendiente".into()),
            (2, 1, "CRISTAL".into(), 2, 5020, "recibido".into()),
        ]
    );
}

#[test]
fn los_trabajos_terminados_no_se_llenan_de_avisos() {
    for estado in ["terminado", "entregado", "anulado"] {
        let conexion = con_trabajo(estado, &[(1, 0, congelado(DOS_CRISTALES))]);
        assert!(materiales(&conexion).is_empty(), "{estado}");
    }
}
