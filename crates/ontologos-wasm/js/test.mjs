import assert from "node:assert/strict";
import test from "node:test";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";
import { readFileSync } from "node:fs";

const __dirname = dirname(fileURLToPath(import.meta.url));
const pkgDir = join(__dirname, "..");

async function loadWasm() {
  const wasmPkg = join(pkgDir, "pkg", "ontologos_wasm.js");
  const wasm = await import(wasmPkg);
  const wasmPath = join(pkgDir, "pkg", "ontologos_wasm_bg.wasm");
  const bytes = readFileSync(wasmPath);
  await wasm.default({ module_or_path: bytes });
  return wasm;
}

test("wasm builder classify el", async () => {
  const { OntologyBuilder, Reasoner, version } = await loadWasm();
  assert.equal(version(), "1.0.1");

  const builder = new OntologyBuilder();
  builder.addClass("http://example.org/Pizza");
  builder.addClass("http://example.org/Food");
  builder.subclassOf("http://example.org/Pizza", "http://example.org/Food");
  const ontology = builder.build();

  const reasoner = new Reasoner(ontology, "el");
  const report = reasoner.classify();
  assert.equal(report.status, "classified");
  assert.ok(report.subsumption_count >= 1);
});

test("wasm fromBytes strict functional syntax", async () => {
  const { Ontology } = await loadWasm();
  const ofn = `Prefix(:=<http://example.org/>)
Ontology(<http://example.org/o>
  Declaration(Class(:A))
  Declaration(Class(:B))
  SubClassOf(:A :B)
)`;
  const ontology = Ontology.fromBytes(new TextEncoder().encode(ofn));
  assert.ok(ontology.axiomCount >= 1);
});

test("wasm typed error on invalid json", async () => {
  const { Ontology } = await loadWasm();
  assert.throws(
    () => Ontology.fromJson("{not json"),
    (err) => err.name === "ParseError" || err.code === "ParseError",
  );
});
