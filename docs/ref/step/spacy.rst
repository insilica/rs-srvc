==========
spacy step
==========

The spacy step runs a Named Entity Recognition model to annotate documents.

The step follows this format:

.. code-block:: yaml

    - uses: github:sysrev/srvc-hello#spacy
      labels: [annotation]

Example ``sr.yaml``:

.. code-block:: yaml

    reviewer: mailto:user@example.com

    labels:
      annotation:
        question: Annotation

    flows:
      spacy:
        steps:
          - run-embedded: remove-reviewed

          - uses: github:sysrev/srvc-hello#spacy
            labels: [annotation]

    sources:
      - step:
          uses: github:insilica/srvc-pubmed-search
          query: angry bees
