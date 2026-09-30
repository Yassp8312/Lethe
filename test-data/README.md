# Datos locales de integración

Esta carpeta se utiliza exclusivamente para pruebas que no involucran memorias USB físicas.

- `lethe.conf`: configuración del objetivo local.
- `local-vault.hc`: contenedor VeraCrypt descartable, ignorado por Git y creado manualmente.

Para crear el contenedor de integración sin exponer una contraseña en argumentos:

1. Abrir `VeraCrypt Format.exe`.
2. Elegir `Create an encrypted file container`.
3. Elegir `Standard VeraCrypt volume`.
4. Usar la ruta absoluta de `test-data/local-vault.hc`.
5. Seleccionar AES y Argon2id.
6. Usar un tamaño de 20 MB y FAT.
7. Introducir una contraseña temporal que no se reutilice fuera de esta prueba.

El contenedor no se versiona, no contiene datos reales y puede eliminarse después de las pruebas.
