using System.Text;
using Xunit;

namespace Ontologos.Tests;

public sealed class SmokeTest
{
    [Fact]
    public void VersionMatchesRelease()
    {
        Assert.Equal("1.1.3", OntologosInfo.Version());
    }

    [Fact]
    public void BuilderClassifyEl()
    {
        using var builder = new OntologyBuilder();
        builder.AddClass("http://example.org/Pizza");
        builder.AddClass("http://example.org/Food");
        builder.SubclassOf("http://example.org/Pizza", "http://example.org/Food");
        using var ontology = builder.Build();
        using var reasoner = new Reasoner(ontology, "el");
        var report = reasoner.Classify();
        Assert.Contains("\"status\":\"classified\"", report);
        Assert.Contains("subsumption_count", report);
    }

    [Fact]
    public void FromBytesStrictFunctionalSyntax()
    {
        const string ofn = """
            Prefix(:=<http://example.org/>)
            Ontology(<http://example.org/o>
              Declaration(Class(:A))
              Declaration(Class(:B))
              SubClassOf(:A :B)
            )
            """;
        using var ontology = Ontology.FromBytes(Encoding.UTF8.GetBytes(ofn));
        Assert.True(ontology.AxiomCount >= 1L);
    }

    [Fact]
    public void SharedOntologyMutationSync()
    {
        using var builder = new OntologyBuilder();
        builder.AddClass("http://example.org/A");
        builder.AddClass("http://example.org/B");
        using var ontology = builder.Build();
        using var reasoner = new Reasoner(ontology, "el");
        Assert.Equal(0L, ontology.AxiomCount);
        reasoner.AddSubclassOf("http://example.org/A", "http://example.org/B");
        Assert.Equal(1L, ontology.AxiomCount);
    }
}
