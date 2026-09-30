import fs from "node:fs/promises";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { Presentation, PresentationFile } from "@oai/artifact-tool";

const workspaceDir = "D:\\Proyectos\\Lethe";
const SKILL_DIR = "C:\\Users\\jorge cintra rad\\.codex\\plugins\\cache\\openai-primary-runtime\\presentations\\26.915.20218\\skills\\presentations";
const TMP_DIR = path.join(workspaceDir, ".build", "pptx");
const FINAL_PPTX = path.join(workspaceDir, "entrega", "Lethe-Final", "Presentacion_Lethe_Final_Rev1.pptx");
const RUNTIME_PYTHON = "C:\\Users\\jorge cintra rad\\.cache\\codex-runtimes\\codex-primary-runtime\\dependencies\\python\\python.exe";

const { resolvePresentationFont, finalizePresentation } = await import(
  pathToFileURL(path.join(SKILL_DIR, "container_tools", "artifact_tool_utils.mjs")).href
);
const font = resolvePresentationFont({ fontFamily: "Aptos" });

const C = {
  navy: "#102E44",
  blue: "#28759B",
  cyan: "#7FC8D8",
  ice: "#EAF3F7",
  orange: "#E78C3A",
  green: "#2D8164",
  red: "#B84C4C",
  ink: "#15232C",
  gray: "#5A6971",
  line: "#BDD0D9",
  white: "#FFFFFF",
};

await fs.mkdir(TMP_DIR, { recursive: true });
await fs.mkdir(path.dirname(FINAL_PPTX), { recursive: true });

const presentation = Presentation.create({ slideSize: { width: 1280, height: 720 } });

function shape(slide, geometry, left, top, width, height, fill, line = "none") {
  return slide.shapes.add({
    geometry,
    position: { left, top, width, height },
    fill,
    line: { style: "solid", fill: line, width: line === "none" ? 0 : 1 },
  });
}

function text(slide, value, left, top, width, height, size = 24, color = C.ink, options = {}) {
  const box = shape(slide, "textbox", left, top, width, height, "none");
  box.text = value;
  box.text.style = {
    typeface: font,
    fontSize: size,
    bold: options.bold ?? false,
    color,
    alignment: options.align ?? "left",
    verticalAlignment: options.valign ?? "top",
    autoFit: "shrinkText",
    wrap: "square",
    insets: options.insets ?? { top: 2, right: 2, bottom: 2, left: 2 },
  };
  return box;
}

function baseSlide(title, number, subtitle = "") {
  const slide = presentation.slides.add();
  slide.background.fill = C.white;
  shape(slide, "rect", 0, 0, 18, 720, C.blue);
  text(slide, title, 66, 42, 1100, 64, 38, C.navy, { bold: true });
  if (subtitle) text(slide, subtitle, 68, 108, 1080, 38, 18, C.gray);
  text(slide, String(number).padStart(2, "0"), 1180, 650, 45, 24, 14, C.gray, { align: "right" });
  return slide;
}

function card(slide, x, y, w, h, titleValue, body, accent = C.blue) {
  shape(slide, "roundRect", x, y, w, h, C.ice, C.line);
  shape(slide, "rect", x, y, 8, h, accent);
  text(slide, titleValue, x + 26, y + 22, w - 48, 38, 22, C.navy, { bold: true });
  text(slide, body, x + 26, y + 68, w - 48, h - 86, 17, C.ink);
}

function bulletList(slide, items, x, y, w, size = 21, gap = 52, color = C.ink) {
  items.forEach((item, index) => {
    shape(slide, "ellipse", x, y + index * gap + 8, 10, 10, C.blue);
    text(slide, item, x + 24, y + index * gap, w - 24, gap - 4, size, color);
  });
}

// 1 Cover
{
  const slide = presentation.slides.add();
  slide.background.fill = C.navy;
  shape(slide, "rect", 0, 0, 24, 720, C.cyan);
  text(slide, "PROYECTO LETHE", 82, 84, 600, 38, 18, C.cyan, { bold: true });
  text(slide, "Almacenamiento cifrado\ncon negación plausible", 80, 148, 820, 170, 54, C.white, { bold: true });
  text(slide, "Diseño, seguridad y cierre del Sprint 7", 84, 348, 720, 44, 25, C.white);
  shape(slide, "roundRect", 83, 458, 620, 96, "#173E57", C.blue);
  text(slide, "Windows · VeraCrypt · Rust\nComponentes gratuitos", 110, 478, 560, 58, 21, C.white);
  shape(slide, "ellipse", 1010, 120, 150, 150, C.blue);
  shape(slide, "ellipse", 1055, 165, 60, 60, C.navy, C.cyan);
  text(slide, "29 SEP 2026", 982, 318, 210, 34, 18, C.cyan, { bold: true, align: "center" });
  slide.speakerNotes.textFrame.setText("Presentación del estado final validado al cierre del Sprint 7.");
}

// 2 Goal
{
  const slide = baseSlide("Una memoria, tres resultados", 2, "La contraseña decide el volumen; Lethe no aprende cuál se abrió");
  card(slide, 70, 190, 350, 300, "A · Contraseña inválida", "No aparece ninguna unidad.\n\nNo hay contador, borrado ni modificación del contenedor.", C.red);
  card(slide, 465, 190, 350, 300, "B · Contraseña señuelo", "VeraCrypt monta el volumen exterior con el contenido preparado para mostrarse.", C.orange);
  card(slide, 860, 190, 350, 300, "C · Contraseña real", "VeraCrypt monta el volumen oculto con los datos privados.", C.green);
  text(slide, "La contraseña se introduce únicamente en el diálogo oficial de VeraCrypt.", 180, 548, 920, 42, 23, C.navy, { bold: true, align: "center" });
  slide.speakerNotes.textFrame.setText("Los tres resultados usan un mismo contenedor con volumen exterior y volumen oculto.");
}

// 3 Architecture
{
  const slide = baseSlide("Arquitectura y límite de confianza", 3, "Lethe coordina; VeraCrypt realiza la criptografía y solicita la contraseña");
  const nodes = [
    [80, "Panel", "Acciones y mensajes"],
    [340, "Núcleo Rust", "Estados y validaciones"],
    [600, "Adaptador", "Invocación segura"],
    [860, "VeraCrypt", "Cifrado y montaje"],
  ];
  nodes.forEach(([x, titleValue, body], index) => {
    shape(slide, "roundRect", x, 230, 220, 150, index === 3 ? C.navy : C.ice, index === 3 ? C.navy : C.line);
    text(slide, titleValue, x + 18, 252, 184, 36, 23, index === 3 ? C.white : C.navy, { bold: true, align: "center" });
    text(slide, body, x + 18, 304, 184, 48, 17, index === 3 ? C.white : C.ink, { align: "center" });
    if (index < nodes.length - 1) shape(slide, "rightArrow", x + 222, 284, 34, 38, C.blue);
  });
  shape(slide, "roundRect", 312, 454, 656, 88, "#FFF4E9", "#F1C28D");
  text(slide, "Límite clave", 338, 475, 150, 30, 20, C.orange, { bold: true });
  text(slide, "Lethe nunca recibe contraseñas ni intenta identificar el volumen abierto.", 482, 470, 450, 44, 20, C.ink);
  slide.speakerNotes.textFrame.setText("La contraseña no pasa por argumentos, archivos, variables de entorno ni registros de Lethe.");
}

// 4 Security decisions
{
  const slide = baseSlide("Decisiones que reducen riesgo", 4);
  const decisions = [
    ["VeraCrypt, no criptografía propia", "Se reutiliza un formato auditado y volúmenes ocultos."],
    ["Solo lectura por defecto", "Reduce el riesgo de dañar el volumen oculto."],
    ["Preparación transaccional", "Un paquete incompleto nunca se publica como válido."],
    ["Pruebas VHDX antes de USB", "Las operaciones destructivas se aislaron en laboratorio."],
  ];
  decisions.forEach(([head, body], i) => {
    const x = i % 2 === 0 ? 72 : 654;
    const y = i < 2 ? 150 : 390;
    card(slide, x, y, 550, 180, head, body, i === 1 ? C.green : C.blue);
  });
  slide.speakerNotes.textFrame.setText("Las decisiones completas se encuentran en docs/DECISIONES.md y en el informe técnico.");
}

// 5 Restore
{
  const slide = baseSlide("Restauración con barreras sucesivas", 5, "El mismo patrón protege VHD/VHDX y la USB física autorizada");
  const steps = [
    ["1", "Plan", "Sin escritura"],
    ["2", "Confirmar", "Objetivo visible"],
    ["3", "Revalidar", "Identidad exacta"],
    ["4", "Elevar", "UAC explícito"],
    ["5", "Verificar", "GPT · exFAT"],
  ];
  steps.forEach(([num, head, body], i) => {
    const x = 72 + i * 232;
    shape(slide, "ellipse", x + 68, 176, 64, 64, i === 4 ? C.green : C.blue);
    text(slide, num, x + 68, 188, 64, 36, 25, C.white, { bold: true, align: "center" });
    text(slide, head, x, 266, 200, 34, 22, C.navy, { bold: true, align: "center" });
    text(slide, body, x, 310, 200, 44, 17, C.gray, { align: "center" });
    if (i < steps.length - 1) shape(slide, "rightArrow", x + 190, 192, 42, 30, C.cyan);
  });
  shape(slide, "roundRect", 150, 425, 980, 114, "#FCEEEE", "#E5B5B5");
  text(slide, "Cualquier diferencia cancela", 185, 449, 360, 34, 24, C.red, { bold: true });
  text(slide, "No se adivina otro objetivo y ningún número de disco se sustituye automáticamente.", 185, 490, 880, 32, 19, C.ink);
  slide.speakerNotes.textFrame.setText("El ayudante elevado vuelve a resolver y verificar el objetivo antes de cualquier escritura.");
}

// 6 Sprint 7
{
  const slide = baseSlide("Qué completa el Sprint 7", 6);
  bulletList(slide, [
    "Restauración física dirigida por una letra explícita",
    "Identidad exacta: etiqueta, bus, capacidad, serial y banderas",
    "Doble confirmación y nueva validación dentro del ayudante elevado",
    "Corrección segura de estados RAW o MBR vacío antes de crear GPT",
    "Reconexión final verificada con la memoria nuevamente convencional",
  ], 92, 160, 1020, 22, 74);
  shape(slide, "roundRect", 765, 560, 410, 68, C.green);
  text(slide, "CICLO FÍSICO COMPLETADO", 782, 578, 376, 30, 18, C.white, { bold: true, align: "center" });
  slide.speakerNotes.textFrame.setText("La contraseña exterior y la oculta se introdujeron directamente en VeraCrypt; Lethe nunca las recibió.");
}

// 7 Device evidence
{
  const slide = baseSlide("JORGITO restaurada y saludable", 7, "Estado observado después de reconectar el 29 de septiembre de 2026");
  const left = [
    ["Letra final", "E:"], ["Etiqueta", "JORGITO"], ["Sistema", "exFAT"], ["Disco", "1 / partición 1"],
  ];
  const right = [
    ["Capacidad", "8 011 120 640 bytes"], ["Bus", "USB"], ["Serial", "057907B76060"], ["Arranque / sistema", "No / No"],
  ];
  [left, right].forEach((rows, col) => rows.forEach(([label, value], row) => {
    const x = col === 0 ? 80 : 660;
    const y = 165 + row * 82;
    text(slide, label, x, y, 210, 28, 17, C.gray, { bold: true });
    text(slide, value, x + 215, y - 2, col === 0 ? 250 : 330, 34, 22, C.navy, { bold: true });
    shape(slide, "line", x, y + 47, 500, 1, "none", C.line);
  }));
  shape(slide, "roundRect", 190, 535, 900, 76, "#EAF6F0", "#A8D2BF");
  text(slide, "GPT  ·  VOLUMEN SALUDABLE  ·  CICLO COMPLETO", 220, 555, 840, 34, 25, C.green, { bold: true, align: "center" });
  slide.speakerNotes.textFrame.setText("La letra y el número de disco cambiaron al reconectar; el serial permaneció estable.");
}

// 8 Test evidence
{
  const slide = baseSlide("Evidencia de calidad", 8);
  shape(slide, "ellipse", 92, 164, 180, 180, C.navy);
  text(slide, "45", 110, 195, 144, 72, 60, C.white, { bold: true, align: "center" });
  text(slide, "pruebas", 110, 268, 144, 32, 20, C.cyan, { align: "center" });
  card(slide, 330, 150, 390, 190, "Compilación", "Release correcto\nClippy sin advertencias", C.green);
  card(slide, 770, 150, 390, 190, "Restauración virtual", "GPT · exFAT · JORGITO\nImagen desmontada al finalizar", C.blue);
  card(slide, 330, 390, 390, 170, "Fallos controlados", "Identidad cambiada, confirmación incorrecta y objetivo externo se cancelan.", C.orange);
  card(slide, 770, 390, 390, 170, "USB física", "Cifrado, montaje, desmontaje, restauración y reconexión aprobados.", C.green);
  text(slide, "Límite", 92, 400, 180, 30, 20, C.red, { bold: true, align: "center" });
  text(slide, "El binario de desarrollo todavía no tiene firma propia.", 80, 442, 205, 92, 17, C.ink, { align: "center" });
  slide.speakerNotes.textFrame.setText("Las pruebas validan las barreras de Lethe; no sustituyen una auditoría de VeraCrypt.");
}

// 9 Final state
{
  const slide = baseSlide("Resultado final", 9, "El objetivo funcional del proyecto quedó validado de extremo a extremo");
  bulletList(slide, [
    "Tres resultados: contraseña inválida, señuelo exterior y volumen oculto",
    "Lethe mantiene las contraseñas fuera de sus procesos y registros",
    "La escritura permanece deshabilitada por defecto",
    "La restauración física cancela ante cualquier diferencia de identidad",
    "Firma de código y auditoría externa quedan como mejoras opcionales",
  ], 92, 158, 1060, 21, 68);
  shape(slide, "roundRect", 180, 545, 920, 78, C.navy);
  text(slide, "JORGITO volvió a E: como GPT / exFAT", 210, 565, 860, 36, 28, C.white, { bold: true, align: "center" });
  slide.speakerNotes.textFrame.setText("La restauración lógica de una memoria flash no se describe como borrado forense.");
}

const requirements = {
  explicitTotalSlideCount: 9,
  requiredNativeTableOwnerSlides: [],
  requiredNativeChartOwnerSlides: [],
};
const fontPolicy = { basis: "design", families: [font] };
const expectedSlideSizeEmu = "12192000,6858000";
const stagingDir = path.join(workspaceDir, ".codex-finalizer");
await fs.mkdir(stagingDir, { recursive: true });
const candidatePath = path.join(stagingDir, "Lethe-Final-Rev1-candidate.pptx");
await (await PresentationFile.exportPptx(presentation)).save(candidatePath);

await finalizePresentation({
  ...requirements,
  workspaceDir,
  candidatePath,
  finalPath: FINAL_PPTX,
  pythonExecutable: RUNTIME_PYTHON,
  integrityValidatorPath: path.join(SKILL_DIR, "container_tools", "inspect_presentation_package_integrity.py"),
  layoutValidatorPath: path.join(SKILL_DIR, "container_tools", "inspect_presentation_layout_geometry.py"),
  layoutArgs: [
    "--expected-slide-size-emu", expectedSlideSizeEmu,
    "--validate-bullet-geometry",
    "--validate-heading-fit",
  ],
  requiredNativeTableOwnerSlides: [],
  fontPolicy,
  verifyArtifactToolImport: true,
  receiptPath: path.join(stagingDir, "Presentacion_Lethe_Final_Rev1.validation.json"),
});

console.log(FINAL_PPTX);
