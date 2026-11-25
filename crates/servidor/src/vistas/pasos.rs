//! Un paso de una ventana del trabajo (cortar, montar…), como se ve en la ficha y en «Mi día».

pub struct PasoEnPantalla {
    pub id: i64,
    pub version: i64,
    pub nombre: String,
    pub estado: String,
    pub quien: String,
}

impl From<&crate::bd::consultas::pasos::FilaPaso> for PasoEnPantalla {
    fn from(p: &crate::bd::consultas::pasos::FilaPaso) -> Self {
        PasoEnPantalla {
            id: p.id,
            version: p.version,
            nombre: p.nombre.clone(),
            estado: p.estado.clone(),
            quien: p.usuario_nombre.clone().unwrap_or_default(),
        }
    }
}
