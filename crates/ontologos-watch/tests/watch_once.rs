//! Integration tests for file-watch reload.

use std::time::Duration;

use ontologos_watch::watch_once;

#[test]
fn watch_once_reloads_on_file_change() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("ontology.owl");
    std::fs::write(
        &path,
        r#"<?xml version="1.0"?>
<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
         xmlns:owl="http://www.w3.org/2002/07/owl#"
         xmlns:rdfs="http://www.w3.org/2000/01/rdf-schema#">
  <owl:Class rdf:about="http://ex.org/A"/>
  <owl:Class rdf:about="http://ex.org/B"/>
  <rdfs:subClassOf>
    <rdf:Description rdf:about="http://ex.org/A">
      <rdfs:subClassOf rdf:resource="http://ex.org/B"/>
    </rdf:Description>
  </rdfs:subClassOf>
</rdf:RDF>"#,
    )
    .expect("write");

    let rx = watch_once(&path, 50).expect("watch_once");
    std::thread::sleep(Duration::from_millis(200));

    std::fs::write(
        &path,
        r#"<?xml version="1.0"?>
<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
         xmlns:owl="http://www.w3.org/2002/07/owl#"
         xmlns:rdfs="http://www.w3.org/2000/01/rdf-schema#">
  <owl:Class rdf:about="http://ex.org/A"/>
  <owl:Class rdf:about="http://ex.org/B"/>
  <owl:Class rdf:about="http://ex.org/C"/>
</rdf:RDF>"#,
    )
    .expect("rewrite");

    let event = rx
        .recv_timeout(Duration::from_secs(5))
        .expect("recv timeout")
        .expect("reload ok");
    assert_eq!(event.path, path);
    assert!(event.ontology.entity_count() >= 2);
}

#[test]
fn watch_once_surfaces_parse_errors() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("bad.owl");
    std::fs::write(&path, "not valid owl").expect("write");

    let rx = watch_once(&path, 50).expect("watch_once");
    std::thread::sleep(Duration::from_millis(200));
    std::fs::write(&path, "still not valid owl content").expect("rewrite");

    let result = rx
        .recv_timeout(Duration::from_secs(5))
        .expect("recv timeout");
    assert!(result.is_err(), "expected parse error, got {result:?}");
}
