use askama::Template;
use dominio::clientes::{EntradaDeCliente, EntradaDeDireccion, ErrorDeCampo};

use super::{GrupoMenu, UsuarioEnPantalla};
use crate::aplicacion::clientes::{FichaDeCliente, ListaDeClientes};
use crate::bd::consultas::clientes::FilaCliente;

/// Errores de un formulario junto a su campo.
#[derive(Debug, Clone, Default)]
pub struct ErroresDeCampo(pub Vec<ErrorDeCampo>);

impl ErroresDeCampo {
    pub fn de(&self, campo: &str) -> Option<&str> {
        self.0
            .iter()
            .find(|e| e.campo == campo)
            .map(|e| e.mensaje.as_str())
    }

    pub fn hay(&self) -> bool {
        !self.0.is_empty()
    }
}

pub struct FilaDeLista {
    pub id: i64,
    pub nombre: String,
    pub sin_nombre: bool,
    pub nif: String,
    pub telefono: String,
    pub localidad: String,
    pub tipo: String,
}

impl From<&FilaCliente> for FilaDeLista {
    fn from(c: &FilaCliente) -> Self {
        FilaDeLista {
            id: c.id,
            nombre: c.nombre_visible(),
            sin_nombre: c.nombre.is_none(),
            nif: c.nif.clone().unwrap_or_default(),
            telefono: c.telefono.clone().unwrap_or_default(),
            localidad: c.fiscal_localidad.clone().unwrap_or_default(),
            tipo: c.tipo.clone(),
        }
    }
}

pub struct VistaListaClientes {
    pub filas: Vec<FilaDeLista>,
    pub texto: String,
    pub archivados: bool,
    pub total: i64,
    pub pagina: i64,
    pub paginas: i64,
}

impl VistaListaClientes {
    pub fn nueva(lista: &ListaDeClientes, texto: String, archivados: bool) -> Self {
        VistaListaClientes {
            filas: lista.clientes.iter().map(FilaDeLista::from).collect(),
            texto,
            archivados,
            total: lista.total,
            pagina: lista.pagina,
            paginas: lista.paginas,
        }
    }

    /// Parámetros de la URL para cambiar de página sin perder la búsqueda.
    pub fn enlace(&self, pagina: &i64) -> String {
        let mut texto = String::new();
        for byte in self.texto.bytes() {
            match byte {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' => {
                    texto.push(char::from(byte))
                }
                otro => texto.push_str(&format!("%{otro:02X}")),
            }
        }
        let archivados = if self.archivados { "&archivados=1" } else { "" };
        format!("/clientes?q={texto}&pagina={pagina}{archivados}")
    }
}

#[derive(Template)]
#[template(path = "clientes.html")]
pub struct PaginaClientes {
    pub usuario: UsuarioEnPantalla,
    pub menu: &'static [GrupoMenu],
    pub seccion: &'static str,
    pub lista: VistaListaClientes,
    pub confirmacion: Option<String>,
    pub deshacer_importacion: Option<i64>,
}

#[derive(Template)]
#[template(path = "clientes_lista.html")]
pub struct FragmentoListaClientes {
    pub lista: VistaListaClientes,
}

pub struct BloqueDireccion {
    pub n: usize,
    pub valores: EntradaDeDireccion,
    pub errores: ErroresDeCampo,
}

#[derive(Template)]
#[template(path = "cliente_formulario.html")]
pub struct PaginaFormularioCliente {
    pub usuario: UsuarioEnPantalla,
    pub menu: &'static [GrupoMenu],
    pub seccion: &'static str,
    pub id: Option<i64>,
    pub version: i64,
    pub valores: EntradaDeCliente,
    pub errores: ErroresDeCampo,
    pub direcciones: Vec<BloqueDireccion>,
    pub misma_direccion: bool,
    pub aviso: Option<String>,
}

impl PaginaFormularioCliente {
    pub fn es_empresa(&self) -> bool {
        self.valores.tipo == "empresa"
    }
}

#[derive(Template)]
#[template(path = "direccion_nueva.html")]
pub struct FragmentoDireccion {
    pub direccion: BloqueDireccion,
}

#[derive(Template)]
#[template(path = "nif_comprobado.html")]
pub struct FragmentoNif {
    pub mensaje: Option<String>,
    pub correcto: bool,
}

pub struct DireccionEnFicha {
    pub id: i64,
    pub resumen: String,
    pub contacto: String,
    pub notas: String,
}

#[derive(Template)]
#[template(path = "cliente_ficha.html")]
pub struct PaginaFichaCliente {
    pub usuario: UsuarioEnPantalla,
    pub menu: &'static [GrupoMenu],
    pub seccion: &'static str,
    pub cliente: FilaCliente,
    pub nombre: String,
    pub direcciones: Vec<DireccionEnFicha>,
    pub nueva_direccion: BloqueDireccion,
    pub aviso: Option<String>,
    pub confirmacion: Option<String>,
}

impl PaginaFichaCliente {
    pub fn direccion_fiscal(&self) -> String {
        let c = &self.cliente;
        [
            &c.fiscal_calle,
            &c.fiscal_cp,
            &c.fiscal_localidad,
            &c.fiscal_provincia,
            &c.fiscal_pais,
        ]
        .into_iter()
        .flatten()
        .cloned()
        .collect::<Vec<_>>()
        .join(", ")
    }
}

pub fn direcciones_en_ficha(ficha: &FichaDeCliente) -> Vec<DireccionEnFicha> {
    ficha
        .direcciones
        .iter()
        .map(|d| DireccionEnFicha {
            id: d.id,
            resumen: d.datos.resumen(),
            contacto: [&d.datos.contacto_nombre, &d.datos.contacto_telefono]
                .into_iter()
                .flatten()
                .cloned()
                .collect::<Vec<_>>()
                .join(" · "),
            notas: d.datos.notas.clone().unwrap_or_default(),
        })
        .collect()
}

#[derive(Template)]
#[template(path = "direccion_editar.html")]
pub struct PaginaEditarDireccion {
    pub usuario: UsuarioEnPantalla,
    pub menu: &'static [GrupoMenu],
    pub seccion: &'static str,
    pub cliente_id: i64,
    pub direccion_id: i64,
    pub nombre_cliente: String,
    pub direccion: BloqueDireccion,
}
