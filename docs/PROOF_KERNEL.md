# Proofer — Proof Kernel Design

## 1. Purpose

The kernel is the trusted component that answers one question:

> Is this proof object a valid proof of this proposition under the active logical rules?

Everything else is allowed to be wrong as long as it eventually produces a kernel-rejected object when wrong.

---

## 2. Kernel Inputs

The kernel must receive elaborated objects, not raw source text.

```text
Theorem
Context
ProofObject
```

---

## 3. Minimal Proof Object

Conceptually:

```rust
enum ProofNode {
    Assumption(FactId),
    ImpIntro { premise: Proposition, body: ProofId },
    ImpElim { function: ProofId, argument: ProofId },
    AndIntro { left: ProofId, right: ProofId },
    AndElimLeft(ProofId),
    AndElimRight(ProofId),
    OrIntroLeft(ProofId),
    OrIntroRight(ProofId),
    ForallIntro { variable: Binder, body: ProofId },
    ForallElim { proof: ProofId, term: Term },
    ExistsIntro { term: Term, proof: ProofId },
    ExistsElim { witness: Binder, proof: ProofId, body: ProofId },
    EqRefl(Term),
    EqSubst { equality: ProofId, proof: ProofId },
}
```

The exact Rust shape may change; the semantic rule set should remain explicit.

---

## 4. Proof Checking

Every proof node has an expected proposition.

The checker recursively verifies:

1. node validity,
2. premise validity,
3. rule preconditions,
4. resulting proposition.

---

## 5. No Trusted Solver

The kernel must not call:

- a geometry solver,
- a SAT solver,
- an SMT solver,
- an algebra system,
- an AI model.

Those systems produce certificates or proof terms.

---

## 6. Geometry Certificate Example

A geometry solver can emit:

```text
SSS(
    equality(AB, AC),
    equality(BM, MC),
    equality(AM, AM)
)
```

The kernel sees the corresponding formal theorem/rule and checks each premise.

---

## 7. Testing Strategy

Kernel tests must contain:

- valid proof objects,
- wrong conclusions,
- missing premises,
- malformed quantifier instantiations,
- invalid substitutions,
- forged geometry certificates.

The most important security property is that malformed certificates cannot be accepted merely because a solver claimed they were valid.

---

## 8. Small Kernel Rule

If a feature increases the amount of trusted code, prefer to implement it as an untrusted elaborator or certificate generator whenever practical.
