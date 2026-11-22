//! Emitir: en una sola transacción se asigna el número correlativo sin huecos (por serie y año),
//! se guarda la fotografía con su SHA-256 y la factura pasa a emitida (BACKEND 5.6). Las señales
//! cobradas en el trabajo pasan a contar en ella.

use dominio::factura::avisos_para_emitir;
use dominio::precio::{LineaParaTotales, calcular_totales};
use dominio::unidades::{Dinero, Porcentaje};
use serde_json::{Value, json};

use super::{FichaDeFactura, cambiar, cargar, exigir_borrador};
use crate::aplicacion::Actor;
use crate::aplicacion::ajustes_presupuesto;
use crate::aplicacion::datos_de_empresa::CabeceraDeEmpresa;
use crate::autenticacion::calcular_sha256;
use crate::bd::consultas::facturas::{self, Emision};
use crate::bd::consultas::{cobros, presupuestos};
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;
use crate::fecha::{anio_de_madrid, fecha_de_madrid, leer_utc};

fn lineas_en_json(ficha: &FichaDeFactura) -> Vec<Value> {
    ficha
        .lineas
        .iter()
        .map(|l| {
            json!({ "descripcion": l.descripcion, "cantidad_mil": l.cantidad_mil,
                "precio_unitario_pu": l.precio_unitario_pu, "descuento_bp": l.descuento_bp,
                "tipo_iva_bp": l.tipo_iva_bp, "base_cent": l.base_cent })
        })
        .collect()
}

fn totales_en_json(ficha: &FichaDeFactura) -> Value {
    let para_totales: Vec<LineaParaTotales> = ficha
        .lineas
        .iter()
        .map(|l| LineaParaTotales {
            importe: Dinero::desde_centimos(l.base_cent),
            tipo_iva: Porcentaje::desde_puntos_basicos(l.tipo_iva_bp),
        })
        .collect();
    let descuento = Porcentaje::desde_puntos_basicos(ficha.factura.descuento_bp);
    let t = calcular_totales(&para_totales, descuento).unwrap_or_default();
    let bases: Vec<Value> = t
        .bases
        .iter()
        .map(|b| {
            json!({ "tipo_bp": b.tipo.puntos_basicos(), "base_cent": b.base.centimos(),
                "cuota_cent": b.cuota.centimos() })
        })
        .collect();
    json!({ "descuento_bp": ficha.factura.descuento_bp, "bases": bases,
        "base_cent": t.base.centimos(), "iva_cent": t.iva.centimos(), "total_cent": t.total.centimos() })
}

/// Lo que se congela: empresa, cliente, líneas, IVA por tipo y totales.
pub(crate) fn fotografia(
    ficha: &FichaDeFactura,
    empresa: &CabeceraDeEmpresa,
    (codigo, fecha): (&str, &str),
) -> String {
    let c = &ficha.cliente;
    let fiscal = [
        &c.fiscal_calle,
        &c.fiscal_cp,
        &c.fiscal_localidad,
        &c.fiscal_provincia,
        &c.fiscal_pais,
    ];
    let direccion: Vec<&String> = fiscal.into_iter().flatten().collect();
    json!({
        "codigo": codigo,
        "fecha": fecha,
        "tipo": ficha.factura.tipo,
        "rectifica": ficha.rectifica.as_ref().map(|r| r.codigo()),
        "motivo_rectificacion": ficha.factura.motivo_rectificacion,
        "empresa": { "titulo": empresa.titulo, "razon_social": empresa.razon_social,
            "lineas": empresa.lineas, "iban": empresa.iban, "pie": empresa.pie },
        "cliente": { "numero": c.id, "nombre": c.nombre, "nif": c.nif, "telefono": c.telefono,
            "email": c.email, "direccion": direccion },
        "lineas": lineas_en_json(ficha),
        "totales": totales_en_json(ficha),
    })
    .to_string()
}

fn comprobar(ficha: &FichaDeFactura, confirmado: bool) -> Result<(), ErrorApp> {
    exigir_borrador(&ficha.factura)?;
    if ficha.lineas.is_empty() {
        return Err(ErrorApp::Validacion("La factura no tiene líneas.".into()));
    }
    let c = &ficha.cliente;
    let avisos = avisos_para_emitir(c.nombre.as_deref(), c.nif.as_deref());
    if !avisos.is_empty() && !confirmado {
        return Err(ErrorApp::Validacion(format!(
            "Antes de emitir: {}. Si es correcto, marca la casilla y emite.",
            avisos.join("; ")
        )));
    }
    Ok(())
}

fn emitir_en(
    tx: &rusqlite::Connection,
    id: i64,
    confirmado: bool,
    ahora: &str,
) -> Result<String, ErrorApp> {
    let ficha = cargar(tx, id)?;
    comprobar(&ficha, confirmado)?;
    let instante = leer_utc(ahora).ok_or_else(|| ErrorApp::Interno("fecha no válida".into()))?;
    let (anio, fecha) = (anio_de_madrid(instante), fecha_de_madrid(instante, 0));
    let serie = ficha.factura.serie.clone();
    let numero = presupuestos::siguiente_numero(tx, &serie, anio)?;
    let codigo = dominio::factura::codigo_de_factura(&serie, anio, Some(numero));
    let textos = ajustes_presupuesto::leer_de_la_base(tx)?.textos;
    let logo = crate::aplicacion::logo_empresa::leer_logo(tx)?;
    let foto = fotografia(
        &ficha,
        &CabeceraDeEmpresa::desde(&textos, logo),
        (&codigo, &fecha),
    );
    let sha = calcular_sha256(&foto);
    let emision = Emision {
        anio,
        numero,
        fecha: &fecha,
        fecha_operacion: &fecha,
        snapshot: &foto,
        sha256: &sha,
    };
    facturas::emitir(tx, id, &emision)?;
    if let (Some(trabajo), "ordinaria") = (ficha.factura.trabajo_id, ficha.factura.tipo.as_str()) {
        cobros::llevar_senales_a_factura(tx, trabajo, id)?;
    }
    Ok(codigo)
}

/// Avisa (sin bloquear) de lo que una factura completa suele exigir; se emite al confirmar.
pub async fn emitir_factura(
    estado: &EstadoApp,
    actor: &Actor,
    id: i64,
    confirmado: bool,
) -> Result<String, ErrorApp> {
    cambiar(
        estado,
        actor,
        ("emitir_factura", "factura"),
        move |tx, ahora| Ok((emitir_en(tx, id, confirmado, ahora)?, id)),
    )
    .await
}
