//! Pruebas de integración en un solo ejecutable: con uno por archivo, cada cambio enlazaba 45
//! programas de ~175 MB (ADR-0035).
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod comun;

mod acabados_incluidos;
mod acceso;
mod apuntes;
mod base_de_datos;
mod catalogo;
mod clientes;
mod clientes_pantallas;
mod copias;
mod copias_pantallas;
mod datos;
mod datos_pantallas;
mod demostracion;
mod demostracion_aislamiento;
mod dorados;
#[cfg(unix)]
mod escritorio;
mod escritorio_pantallas;
mod escritorio_recuperacion;
mod factura_impresa;
mod facturas;
mod facturas_pantallas;
mod facturas_por_cobrar;
mod herrajes_y_marcas;
mod importar_clientes;
mod logo_empresa;
mod medidas_habituales;
mod mosquitera;
mod parteluces;
mod permisos;
mod permisos_pantallas;
mod persiana;
mod persiana_pared;
mod presupuestos;
mod presupuestos_caducidad;
mod presupuestos_pantallas;
mod series;
mod series_incluidas;
mod siluetas;
mod stock;
mod stock_pantallas;
mod taller;
mod tarifas;
mod trabajos;
mod trabajos_pantallas;
