-- Los presupuestos caducan a los 30 días porque los precios cambian (propietario, 2026-11-23;
-- ADR-0027). Si la validez estaba vacía («sin fecha», lo de antes por defecto), pasa a 30; un
-- número que ya hubiera puesto el propietario no se toca. No cambia ningún presupuesto: solo la
-- validez de los que se hagan o se renueven desde ahora.
UPDATE configuracion
SET valor = '30', actualizado_en = strftime('%Y-%m-%dT%H:%M:%SZ', 'now'), usuario_id = NULL
WHERE clave = 'presupuesto.validez_dias' AND trim(valor) = '';
