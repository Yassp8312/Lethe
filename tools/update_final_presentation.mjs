import crypto from "node:crypto";
import fs from "node:fs/promises";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { FileBlob, PresentationFile } from "@oai/artifact-tool";

const workspaceDir = "D:\\Proyectos\\Lethe";
const SKILL_DIR = "C:\\Users\\jorge cintra rad\\.codex\\plugins\\cache\\openai-primary-runtime\\presentations\\26.915.20218\\skills\\presentations";
const RUNTIME_PYTHON = "C:\\Users\\jorge cintra rad\\.cache\\codex-runtimes\\codex-primary-runtime\\dependencies\\python\\python.exe";
const sourcePath = path.join(workspaceDir, "entrega", "Lethe-Final", "Presentacion_Lethe_Final_Rev1.pptx");
const finalPath = path.join(workspaceDir, "entrega", "Lethe-Final", "Presentacion_Lethe_Final.pptx");
const stagingDir = path.join(workspaceDir, ".codex-finalizer");
const candidatePath = path.join(stagingDir, "Lethe-Final-candidate.pptx");

const { finalizePresentation } = await import(
  pathToFileURL(path.join(SKILL_DIR, "container_tools", "artifact_tool_utils.mjs")).href
);

const presentation = await PresentationFile.importPptx(await FileBlob.load(sourcePath));

const replacements = [
  ["sh/7qp4be9c", "Diseño, seguridad y cierre del Sprint 7", "Diseño, seguridad y cierre final del proyecto"],
  ["sh/ts7md4r2", "Windows · VeraCrypt · Rust\nComponentes gratuitos", "Windows, VeraCrypt y Rust\nComponentes gratuitos"],
  ["sh/ozy1ofad", "La contraseña decide el volumen; Lethe no aprende cuál se abrió", "La contraseña decide el volumen sin revelar a Lethe cuál se abrió"],
  ["sh/w3i1sfa9", "A · Contraseña inválida", "Caso A  Contraseña inválida"],
  ["sh/9wnqhczy", "B · Contraseña señuelo", "Caso B  Contraseña señuelo"],
  ["sh/xk7qlczu", "C · Contraseña real", "Caso C  Contraseña real"],
  ["sh/d0jax03i", "Lethe coordina; VeraCrypt realiza la criptografía y solicita la contraseña", "Lethe coordina y VeraCrypt realiza la criptografía y solicita la contraseña"],
  ["sh/yhg7epsj", "Qué completa el Sprint 7", "Cierre del producto"],
  ["sh/98rqt4r6", "Restauración física dirigida por una letra explícita", "Panel con estado real, actualización manual y acceso directo a L:"],
  ["sh/7m98ru9g", "Identidad exacta: etiqueta, bus, capacidad, serial y banderas", "Desmontaje verificado aunque VeraCrypt no refresque su ventana"],
  ["sh/pc76hkr2", "Doble confirmación y nueva validación dentro del ayudante elevado", "Redetección de JORGITO cuando Windows cambia la letra"],
  ["sh/w32twb6l", "Corrección segura de estados RAW o MBR vacío antes de crear GPT", "Lanzador desacoplado de la USB y rutas normalizadas"],
  ["sh/u1kbu1ov", "Reconexión final verificada con la memoria nuevamente convencional", "Release final sincronizado en staging y en la memoria"],
  ["sh/47mt0b6x", "CICLO FÍSICO COMPLETADO", "RELEASE FINAL VALIDADO"],
  ["sh/cb2tkvap", "JORGITO restaurada y saludable", "JORGITO preparada para uso cifrado"],
  ["sh/dcbud0ra", "Estado observado después de reconectar el 29 de septiembre de 2026", "Estado final validado el 29 de septiembre de 2026"],
  ["sh/zedcfa9g", "Letra final", "Letra física"],
  ["sh/n6x0fqlo", "Disco", "Contenedor"],
  ["sh/25ojml43", "1 / partición 1", "vault.hc"],
  ["sh/o761ov2t", "Capacidad", "Capacidad física"],
  ["sh/f2d0b6lc", "8 011 120 640 bytes", "8,01 GB"],
  ["sh/id4ju1oz", "Bus", "Tamaño contenedor"],
  ["sh/jedkn654", "USB", "6 GiB"],
  ["sh/50v2pgna", "Serial", "Unidad cifrada"],
  ["sh/q143ylov", "057907B76060", "L:"],
  ["sh/s3ml0v61", "Arranque / sistema", "Estado"],
  ["sh/d4f2t0nm", "No / No", "Disponible"],
  ["sh/p0zmlk7u", "GPT  ·  VOLUMEN SALUDABLE  ·  CICLO COMPLETO", "RELEASE FINAL / CONTENEDOR LISTO / PRUEBAS APROBADAS"],
  ["sh/o7ydoret", "Release correcto\nClippy sin advertencias", "Release final\n45 pruebas aprobadas"],
  ["sh/0jydkreh", "GPT · exFAT · JORGITO\nImagen desmontada al finalizar", "GPT / exFAT / JORGITO\nImagen desmontada al finalizar"],
  ["sh/kfixwbe1", "Identidad cambiada, confirmación incorrecta y objetivo externo se cancelan.", "Rutas inválidas y cambios de letra se controlan sin cerrar el panel."],
  ["sh/7u5w765k", "Cifrado, montaje, desmontaje, restauración y reconexión aprobados.", "Tres casos de acceso, desmontaje, reconexión y release aprobado."],
  ["sh/0b65obm9", "El objetivo funcional del proyecto quedó validado de extremo a extremo", "El release final quedó validado de extremo a extremo"],
  ["sh/2xsjepgb", "La restauración física cancela ante cualquier diferencia de identidad", "El panel consulta el montaje real y permite desmontarlo"],
  ["sh/cnu1kzyd", "Firma de código y auditoría externa quedan como mejoras opcionales", "Firma de código y auditoría externa permanecen como mejoras opcionales"],
  ["sh/qlcjipgn", "JORGITO volvió a E: como GPT / exFAT", "JORGITO LISTA PARA USO CIFRADO"],
];

for (const [id, oldText, newText] of replacements) {
  const target = presentation.resolve(id);
  target.text.replace(oldText, newText);
}

presentation.slides.getItem(0).speakerNotes.textFrame.setText(
  "Presentación del release final validado el 29 de septiembre de 2026."
);
presentation.slides.getItem(5).speakerNotes.textFrame.setText(
  "El cierre corrigió el panel, el lanzador y la sincronización del release sin modificar las contraseñas."
);
presentation.slides.getItem(6).speakerNotes.textFrame.setText(
  "La letra física puede cambiar. Lethe vuelve a detectar una única carpeta válida y mantiene L: como unidad cifrada."
);
presentation.slides.getItem(7).speakerNotes.textFrame.setText(
  "Las pruebas automáticas y la aceptación manual cubren el release final."
);
presentation.slides.getItem(8).speakerNotes.textFrame.setText(
  "La entrega final conserva como límites la ausencia de firma propia y de auditoría externa."
);

await fs.mkdir(stagingDir, { recursive: true });
await fs.mkdir(path.dirname(finalPath), { recursive: true });
await (await PresentationFile.exportPptx(presentation)).save(candidatePath);

const sourceSha256 = crypto.createHash("sha256").update(await fs.readFile(sourcePath)).digest("hex");
await finalizePresentation({
  explicitTotalSlideCount: 9,
  requiredNativeTableOwnerSlides: [],
  requiredNativeChartOwnerSlides: [],
  workspaceDir,
  candidatePath,
  finalPath,
  pythonExecutable: RUNTIME_PYTHON,
  integrityValidatorPath: path.join(SKILL_DIR, "container_tools", "inspect_presentation_package_integrity.py"),
  layoutValidatorPath: path.join(SKILL_DIR, "container_tools", "inspect_presentation_layout_geometry.py"),
  layoutArgs: [
    "--expected-slide-size-emu", "12192000,6858000",
    "--validate-bullet-geometry",
    "--validate-heading-fit",
  ],
  fontPolicy: {
    basis: "reference",
    families: ["Aptos"],
    referencePath: sourcePath,
    referenceSha256: sourceSha256,
  },
  verifyArtifactToolImport: true,
  receiptPath: path.join(stagingDir, "Presentacion_Lethe_Final_Rev2.validation.json"),
});

console.log(finalPath);
