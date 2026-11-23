use super::*;

const AHORA: &str = "2026-11-21T10:00:00Z";

fn preparada() -> Connection {
    let mut conexion = Connection::open_in_memory().unwrap();
    preparar_esquema(&mut conexion, AHORA).unwrap();
    conexion
}

fn version(conexion: &Connection) -> u32 {
    conexion
        .query_row("PRAGMA user_version", [], |f| f.get(0))
        .unwrap()
}

fn anotar_dato(conexion: &Connection) {
    conexion
        .execute(
            "INSERT INTO configuracion (clave, valor, actualizado_en) VALUES ('k', 'v', ?1)",
            [AHORA],
        )
        .unwrap();
}

fn dato(conexion: &Connection) -> String {
    conexion
        .query_row(
            "SELECT valor FROM configuracion WHERE clave = 'k'",
            [],
            |f| f.get(0),
        )
        .unwrap()
}

#[test]
fn la_migracion_0001_publicada_no_se_edita() {
    // Sin `\r`: en Windows Git puede sacar el archivo con finales CRLF y el SQL es el mismo.
    let sin_retornos = MIGRACIONES[0].sql.replace('\r', "");
    assert_eq!(calcular_sha256(&sin_retornos), HUELLA_DE_LA_0001);
    let en_windows = sin_retornos.replace('\n', "\r\n");
    assert_eq!(calcular_sha256(&en_windows), HUELLA_DE_LA_0001_EN_WINDOWS);
    for (indice, migracion) in MIGRACIONES.iter().enumerate() {
        assert_eq!(
            migracion.numero as usize,
            indice + 1,
            "numeradas sin huecos"
        );
    }
}

#[test]
fn crea_todas_las_tablas_en_una_base_vacia() {
    let conexion = preparada();
    assert_eq!(version(&conexion), version_del_programa());
    for tabla in [
        "auditoria",
        "esquema",
        "serie",
        "usuario",
        "factura",
        "cobro",
    ] {
        let existe: bool = conexion
            .query_row(
                "SELECT EXISTS (SELECT 1 FROM sqlite_schema WHERE name = ?1)",
                [tabla],
                |f| f.get(0),
            )
            .unwrap();
        assert!(existe, "{tabla}");
    }
    let integridad: String = conexion
        .query_row("PRAGMA integrity_check", [], |f| f.get(0))
        .unwrap();
    assert_eq!(integridad, "ok");
}

#[test]
fn preparar_dos_veces_no_cambia_nada() {
    let mut conexion = preparada();
    anotar_dato(&conexion);
    assert!(preparar_esquema(&mut conexion, AHORA).is_ok());
    assert_eq!(dato(&conexion), "v");
}

fn base_antigua(huella: &str) -> Connection {
    // Así crearon la base las versiones 0.8.0 a 0.10.0: esquema único, huella y sin número.
    let antigua = Connection::open_in_memory().unwrap();
    antigua.execute_batch(MIGRACIONES[0].sql).unwrap();
    antigua
        .execute(
            "INSERT INTO esquema (huella, creado_en) VALUES (?1, ?2)",
            params![huella, AHORA],
        )
        .unwrap();
    anotar_dato(&antigua);
    antigua
}

#[test]
fn adopta_las_bases_de_las_versiones_sin_migraciones_con_sus_datos() {
    for huella in [HUELLA_DE_LA_0001, HUELLA_DE_LA_0001_EN_WINDOWS] {
        let mut antigua = base_antigua(huella);
        assert_eq!(version(&antigua), 0);
        preparar_esquema(&mut antigua, AHORA).unwrap();
        assert_eq!(version(&antigua), 1, "{huella}");
        assert_eq!(dato(&antigua), "v");
    }
}

#[test]
fn rechaza_bases_desconocidas_o_mas_nuevas() {
    let mut otra = preparada();
    otra.pragma_update(None, "user_version", 0).unwrap();
    otra.execute("UPDATE esquema SET huella = ?1", [&"0".repeat(64)])
        .unwrap();
    assert!(matches!(
        preparar_esquema(&mut otra, AHORA),
        Err(ErrorEsquema::Desconocida)
    ));
    let mut suelta = Connection::open_in_memory().unwrap();
    suelta
        .execute_batch("CREATE TABLE version_esquema (version INTEGER)")
        .unwrap();
    assert!(matches!(
        preparar_esquema(&mut suelta, AHORA),
        Err(ErrorEsquema::Desconocida)
    ));
    let mut nueva = preparada();
    nueva.pragma_update(None, "user_version", 99).unwrap();
    assert!(matches!(
        preparar_esquema(&mut nueva, AHORA),
        Err(ErrorEsquema::MasNueva { base: 99, .. })
    ));
}

const AMPLIAR: Migracion = Migracion {
    numero: 2,
    sql: "ALTER TABLE configuracion ADD COLUMN nota TEXT;",
};

#[test]
fn una_migracion_nueva_conserva_los_datos() {
    let mut conexion = preparada();
    anotar_dato(&conexion);
    let migraciones = [MIGRACIONES[0], AMPLIAR];
    assert_eq!(migraciones_pendientes(&conexion).unwrap(), 0);
    aplicar_migraciones(&mut conexion, &migraciones, AHORA).unwrap();
    assert_eq!(version(&conexion), 2);
    assert_eq!(dato(&conexion), "v");
    let nota: Option<String> = conexion
        .query_row(
            "SELECT nota FROM configuracion WHERE clave = 'k'",
            [],
            |f| f.get(0),
        )
        .unwrap();
    assert_eq!(nota, None);
    assert_eq!(
        leer_huella_guardada(&conexion).unwrap().unwrap(),
        calcular_sha256("migracion-2")
    );
    let mut fallos = conexion.prepare("PRAGMA foreign_key_check").unwrap();
    assert!(fallos.query([]).unwrap().next().unwrap().is_none());
}

#[test]
fn una_migracion_que_falla_no_deja_nada_a_medias() {
    let mut conexion = preparada();
    anotar_dato(&conexion);
    let rota = Migracion {
        numero: 2,
        sql: "ALTER TABLE configuracion ADD COLUMN nota TEXT; ALTER TABLE no_existe ADD COLUMN x TEXT;",
    };
    let migraciones = [MIGRACIONES[0], rota];
    assert!(aplicar_migraciones(&mut conexion, &migraciones, AHORA).is_err());
    assert_eq!(version(&conexion), 1);
    assert!(conexion.prepare("SELECT nota FROM configuracion").is_err());
    assert_eq!(dato(&conexion), "v");
}

#[test]
fn la_auditoria_y_las_versiones_de_serie_no_se_modifican() {
    let conexion = preparada();
    conexion
        .execute(
            "INSERT INTO auditoria (cuando_en, accion, entidad) VALUES (?1, 'prueba', 'x')",
            [AHORA],
        )
        .unwrap();
    assert!(
        conexion
            .execute("UPDATE auditoria SET accion = 'otra'", [])
            .is_err()
    );
    assert!(conexion.execute("DELETE FROM auditoria", []).is_err());
    conexion
        .execute_batch(
            "INSERT INTO serie (codigo, fabricante, creada_en) VALUES ('S', 'F', 'x');
             INSERT INTO serie_version (serie_id, numero, estado, origen_yaml, sha256, avisos, creada_en)
             VALUES (1, 1, 'borrador', 'serie: S', printf('%064d', 0), 0, 'x');",
        )
        .unwrap();
    assert!(
        conexion
            .execute("UPDATE serie_version SET avisos = 1", [])
            .is_err()
    );
    assert!(conexion.execute("DELETE FROM serie_version", []).is_err());
}
