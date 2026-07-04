namespace Ontologos;

/// <summary>Entailment check input (exactly one axiom shape).</summary>
public sealed class EntailmentCheck
{
    public string? Sub { get; init; }
    public string? Sup { get; init; }
    public string? Individual { get; init; }
    public string? ClassIri { get; init; }
    public string? Subject { get; init; }
    public string? Property { get; init; }
    public string? Object { get; init; }
}
