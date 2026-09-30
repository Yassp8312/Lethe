import { FileBlob, PresentationFile } from "@oai/artifact-tool";

const sourcePath = "D:\\Proyectos\\Lethe\\entrega\\Lethe-Final\\Presentacion_Lethe_Final_Rev1.pptx";
const presentation = await PresentationFile.importPptx(await FileBlob.load(sourcePath));
const snapshot = await presentation.inspect({
  kind: "slide,textbox,shape,notes,layout",
  maxChars: 50000,
});
console.log(snapshot.ndjson);
