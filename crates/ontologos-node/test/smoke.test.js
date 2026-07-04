const test = require("node:test");
const assert = require("node:assert/strict");
const path = require("node:path");
const fs = require("node:fs");
const os = require("node:os");

const {
  Ontology,
  OntologyBuilder,
  Reasoner,
  version,
  errorCodeFromMessage,
} = require("..");

test("version", () => {
  assert.equal(version(), "1.0.1");
});

test("builder classify el", () => {
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

test("load from bytes strict", () => {
  const ofn = `Prefix(:=<http://example.org/>)
Ontology(<http://example.org/o>
  Declaration(Class(:A))
  Declaration(Class(:B))
  SubClassOf(:A :B)
)`;
  const ontology = Ontology.fromBytes(Buffer.from(ofn, "utf8"));
  assert.ok(ontology.axiomCount >= 1);
});

test("load family owl from path", () => {
  const family = path.join(
    __dirname,
    "..",
    "..",
    "..",
    "benchmarks",
    "data",
    "family.owl",
  );
  const reasoner = Reasoner.fromPath(family, "rl");
  const report = reasoner.classify();
  assert.equal(report.status, "classified");
});

test("isEntailed subclass chain", () => {
  const builder = new OntologyBuilder();
  builder.addClass("http://example.org/A");
  builder.addClass("http://example.org/B");
  builder.addClass("http://example.org/C");
  builder.subclassOf("http://example.org/A", "http://example.org/B");
  builder.subclassOf("http://example.org/B", "http://example.org/C");
  const ontology = builder.build();
  const reasoner = new Reasoner(ontology, "el");
  assert.equal(
    reasoner.isEntailed({
      sub: "http://example.org/A",
      sup: "http://example.org/C",
    }),
    true,
  );
});

test("checkConsistency", () => {
  const builder = new OntologyBuilder();
  builder.addClass("http://example.org/A");
  const reasoner = new Reasoner(builder.build(), "el");
  const result = reasoner.checkConsistency();
  assert.equal(result.consistent, true);
  assert.equal(result.complete, true);
});

test("shared ontology mutation sync", () => {
  const builder = new OntologyBuilder();
  builder.addClass("http://example.org/A");
  builder.addClass("http://example.org/B");
  const ontology = builder.build();
  assert.equal(ontology.axiomCount, 0);

  const reasoner = new Reasoner(ontology, "el");
  reasoner.addSubclassOf("http://example.org/A", "http://example.org/B");
  assert.equal(ontology.axiomCount, 1);
});

test("loadIn sandbox", () => {
  const base = fs.mkdtempSync(path.join(os.tmpdir(), "ontologos-node-"));
  const ofn = `Prefix(:=<http://example.org/>)
Ontology(<http://example.org/o>
  Declaration(Class(:A))
  Declaration(Class(:B))
  SubClassOf(:A :B)
)`;
  const file = path.join(base, "test.ofn");
  fs.writeFileSync(file, ofn);
  const ontology = Ontology.loadIn(base, "test.ofn");
  assert.ok(ontology.axiomCount >= 1);
});

test("Reasoner.loadIn", () => {
  const base = fs.mkdtempSync(path.join(os.tmpdir(), "ontologos-reasoner-"));
  const ofn = `Prefix(:=<http://example.org/>)
Ontology(<http://example.org/o>
  Declaration(Class(:A))
  Declaration(Class(:B))
  SubClassOf(:A :B)
)`;
  fs.writeFileSync(path.join(base, "test.ofn"), ofn);
  const reasoner = Reasoner.loadIn(base, "test.ofn", "el");
  const report = reasoner.classify();
  assert.equal(report.status, "classified");
});

test("error code helper", () => {
  assert.equal(errorCodeFromMessage("ParseError: bad json"), "ParseError");
  assert.equal(errorCodeFromMessage("something else"), null);
});
