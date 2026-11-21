pub mod acabados;
pub mod articulos;
pub mod auditoria;
pub mod bloqueo_ip;
pub mod clientes;
pub mod configuracion;
pub mod direcciones;
pub mod importaciones;
pub mod proveedores;
pub mod series;
pub mod sesiones;
pub mod tarifas;
pub mod usuarios;

/// Escapa los comodines de LIKE para que «%» o «_» se busquen como texto.
pub fn escapar_like(texto: &str) -> String {
    texto
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}
