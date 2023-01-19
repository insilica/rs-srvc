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
        type: webannotation
        question: Annotation

    flows:
      spacy:
        steps:
          - uses: github:insilica/srvc-pubmed-search
            query: angry bees

          - run-embedded: remove-reviewed

          - uses: github:sysrev/srvc-hello#spacy
            labels: [annotation]
