package dev.ontologos;

/** Entailment check input (exactly one axiom shape). */
public final class EntailmentCheck {
    private final String sub;
    private final String sup;
    private final String individual;
    private final String classIri;
    private final String subject;
    private final String property;
    private final String object;

    private EntailmentCheck(Builder builder) {
        this.sub = builder.sub;
        this.sup = builder.sup;
        this.individual = builder.individual;
        this.classIri = builder.classIri;
        this.subject = builder.subject;
        this.property = builder.property;
        this.object = builder.object;
    }

    public static Builder builder() {
        return new Builder();
    }

    String sub() {
        return sub;
    }

    String sup() {
        return sup;
    }

    String individual() {
        return individual;
    }

    String classIri() {
        return classIri;
    }

    String subject() {
        return subject;
    }

    String property() {
        return property;
    }

    String object() {
        return object;
    }

    /** Builder for {@link EntailmentCheck}. */
    public static final class Builder {
        private String sub;
        private String sup;
        private String individual;
        private String classIri;
        private String subject;
        private String property;
        private String object;

        public Builder sub(String sub) {
            this.sub = sub;
            return this;
        }

        public Builder sup(String sup) {
            this.sup = sup;
            return this;
        }

        public Builder individual(String individual) {
            this.individual = individual;
            return this;
        }

        public Builder classIri(String classIri) {
            this.classIri = classIri;
            return this;
        }

        public Builder subject(String subject) {
            this.subject = subject;
            return this;
        }

        public Builder property(String property) {
            this.property = property;
            return this;
        }

        public Builder object(String object) {
            this.object = object;
            return this;
        }

        public EntailmentCheck build() {
            return new EntailmentCheck(this);
        }
    }
}
