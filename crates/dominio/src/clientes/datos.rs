use super::telefono::normalizar_telefono;
use crate::nif::Nif;
use crate::texto::{limpiar_opcional, normalizar_para_buscar};

const LARGO_CAMPO: usize = 200;
const LARGO_NOTAS: usize = 2_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TipoCliente {
    #[default]
    Particular,
    Empresa,
}

impl TipoCliente {
    pub fn desde_texto(texto: &str) -> Option<TipoCliente> {
        match normalizar_para_buscar(texto).as_str() {
            "" | "particular" => Some(TipoCliente::Particular),
            "empresa" => Some(TipoCliente::Empresa),
            _ => None,
        }
    }

    pub fn como_texto(self) -> &'static str {
        match self {
            TipoCliente::Particular => "particular",
            TipoCliente::Empresa => "empresa",
        }
    }
}

/// Lo que llega del formulario o de una fila del CSV, tal cual se escribió.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EntradaDeCliente {
    pub tipo: String,
    pub nombre: String,
    pub nif: String,
    pub email: String,
    pub telefono: String,
    pub fiscal_calle: String,
    pub fiscal_cp: String,
    pub fiscal_localidad: String,
    pub fiscal_provincia: String,
    pub fiscal_pais: String,
    pub forma_de_pago: String,
    pub notas: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DatosDeCliente {
    pub tipo: TipoCliente,
    pub nombre: Option<String>,
    pub nif: Option<Nif>,
    pub email: Option<String>,
    pub telefono: Option<String>,
    pub fiscal_calle: Option<String>,
    pub fiscal_cp: Option<String>,
    pub fiscal_localidad: Option<String>,
    pub fiscal_provincia: Option<String>,
    pub fiscal_pais: Option<String>,
    pub forma_de_pago: Option<String>,
    pub notas: Option<String>,
}

impl DatosDeCliente {
    /// Lo que encuentra la búsqueda de clientes, sin tildes ni mayúsculas.
    pub fn texto_de_busqueda(&self) -> String {
        let partes = [
            self.nombre.as_deref(),
            self.nif.as_ref().map(Nif::como_texto),
            self.telefono.as_deref(),
            self.email.as_deref(),
            self.fiscal_localidad.as_deref(),
        ];
        normalizar_para_buscar(&partes.into_iter().flatten().collect::<Vec<_>>().join(" "))
    }
}

/// Un problema junto al campo donde se escribió, para mostrarlo al lado.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorDeCampo {
    pub campo: &'static str,
    pub mensaje: String,
}

pub fn validar_cliente(entrada: &EntradaDeCliente) -> Result<DatosDeCliente, Vec<ErrorDeCampo>> {
    let mut errores = Vec::new();
    let tipo = TipoCliente::desde_texto(&entrada.tipo).unwrap_or_else(|| {
        errores.push(error("tipo", "el tipo debe ser «particular» o «empresa»"));
        TipoCliente::Particular
    });
    let mut texto = |campo: &'static str, valor: &str, largo: usize| {
        let limpio = limpiar_opcional(valor);
        if limpio.as_ref().is_some_and(|v| v.chars().count() > largo) {
            errores.push(error(
                campo,
                &format!("no puede tener más de {largo} caracteres"),
            ));
        }
        limpio
    };
    let mut datos = DatosDeCliente {
        tipo,
        nombre: texto("nombre", &entrada.nombre, LARGO_CAMPO),
        nif: None,
        email: texto("email", &entrada.email, LARGO_CAMPO),
        telefono: texto("telefono", &entrada.telefono, 30),
        fiscal_calle: texto("fiscal_calle", &entrada.fiscal_calle, LARGO_CAMPO),
        fiscal_cp: texto("fiscal_cp", &entrada.fiscal_cp, 10),
        fiscal_localidad: texto("fiscal_localidad", &entrada.fiscal_localidad, LARGO_CAMPO),
        fiscal_provincia: texto("fiscal_provincia", &entrada.fiscal_provincia, LARGO_CAMPO),
        fiscal_pais: texto("fiscal_pais", &entrada.fiscal_pais, LARGO_CAMPO),
        forma_de_pago: texto("forma_de_pago", &entrada.forma_de_pago, LARGO_CAMPO),
        notas: texto("notas", &entrada.notas, LARGO_NOTAS),
    };
    comprobar_formatos(entrada, &mut datos, &mut errores);
    if errores.is_empty() {
        Ok(datos)
    } else {
        Err(errores)
    }
}

fn comprobar_formatos(
    entrada: &EntradaDeCliente,
    datos: &mut DatosDeCliente,
    errores: &mut Vec<ErrorDeCampo>,
) {
    datos.nif = validar_nif(&entrada.nif, errores);
    comprobar_email(datos.email.as_deref(), errores);
    datos.telefono = comprobar_telefono(
        "telefono",
        datos.telefono.take(),
        datos.fiscal_pais.as_deref(),
        errores,
    );
}

fn error(campo: &'static str, mensaje: &str) -> ErrorDeCampo {
    ErrorDeCampo {
        campo,
        mensaje: mensaje.to_owned(),
    }
}

fn validar_nif(texto: &str, errores: &mut Vec<ErrorDeCampo>) -> Option<Nif> {
    let limpio = limpiar_opcional(texto)?;
    match Nif::desde_texto(&limpio) {
        Ok(nif) => Some(nif),
        Err(motivo) => {
            errores.push(error("nif", &motivo.to_string()));
            None
        }
    }
}

fn comprobar_email(email: Option<&str>, errores: &mut Vec<ErrorDeCampo>) {
    let Some(email) = email else { return };
    let valido = email.split_once('@').is_some_and(|(usuario, dominio)| {
        !usuario.is_empty() && dominio.contains('.') && !dominio.contains('@')
    }) && !email.contains(char::is_whitespace);
    if !valido {
        errores.push(error(
            "email",
            "el correo no parece válido (ejemplo: nombre@correo.es)",
        ));
    }
}

pub(super) fn comprobar_telefono(
    campo: &'static str,
    telefono: Option<String>,
    pais: Option<&str>,
    errores: &mut Vec<ErrorDeCampo>,
) -> Option<String> {
    let escrito = telefono?;
    match normalizar_telefono(&escrito, pais) {
        Ok(normalizado) => Some(normalizado),
        Err(motivo) => {
            errores.push(error(campo, motivo));
            Some(escrito)
        }
    }
}

/// El alta y la edición a mano exigen nombre y teléfono, para no tener fichas
/// con solo un nombre (decisión del propietario, 2026-11-22). La importación no.
pub fn validar_alta_de_cliente(
    entrada: &EntradaDeCliente,
) -> Result<DatosDeCliente, Vec<ErrorDeCampo>> {
    let resultado = validar_cliente(entrada);
    let mut errores = resultado.as_ref().err().cloned().unwrap_or_default();
    let falta = |campo: &str| limpiar_opcional(campo).is_none();
    if falta(&entrada.nombre) {
        errores.push(error("nombre", "escribe el nombre o la razón social"));
    }
    if falta(&entrada.telefono) {
        errores.push(error("telefono", "escribe un teléfono de contacto"));
    }
    if errores.is_empty() {
        resultado
    } else {
        Err(errores)
    }
}

/// Un cliente sin nombre se identifica por su número (PANTALLAS, apartado 5).
pub fn nombre_visible(id: i64, nombre: Option<&str>) -> String {
    match nombre.map(str::trim).filter(|n| !n.is_empty()) {
        Some(nombre) => nombre.to_owned(),
        None => format!("Cliente sin nombre (n.º {id})"),
    }
}
