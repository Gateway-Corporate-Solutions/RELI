# RELI Deterministic Fixtures

This directory stores deterministic fixture inputs and expected outputs for algorithm profiles.

Rules:

1. Every fixture must include profile id, input payload hash, and expected output hash.
2. Expected confidence evidence must be versioned with schema version.
3. Fixtures are immutable once referenced by a release tag.
4. New fixture revisions must use a new fixture id.

Initial profiles:

- viterbi-basic-bsc
- ldpc-minsum-basic
- turbo-basic

Additional scenarios:

- ldpc-minsum-adversarial-lowquality (high residual noise case)
