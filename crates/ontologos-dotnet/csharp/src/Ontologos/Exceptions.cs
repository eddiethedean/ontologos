namespace Ontologos;

/// <summary>Base exception for OntoLogos binding failures.</summary>
public class OntologosException : Exception
{
    public OntologosException(string message)
        : base(message)
    {
    }
}

/// <summary>Parse or validation failure.</summary>
public sealed class ParseException : OntologosException
{
    public ParseException(string message)
        : base(message)
    {
    }
}

/// <summary>Resource limit exceeded.</summary>
public sealed class ResourceLimitException : OntologosException
{
    public ResourceLimitException(string message)
        : base(message)
    {
    }
}

/// <summary>Reasoning stopped before completion.</summary>
public sealed class IncompleteReasoningException : OntologosException
{
    public IncompleteReasoningException(string message)
        : base(message)
    {
    }
}

/// <summary>Conflicting ontology mutation.</summary>
public sealed class OntologyConflictException : OntologosException
{
    public OntologyConflictException(string message)
        : base(message)
    {
    }
}
