====================
remove-reviewed step
====================

This step skips documents that have already been assigned answers by the reviewer.
The step follows this format:

.. code-block:: yaml

      - run-embedded: remove-reviewed

Example ``sr.yaml``:

.. code-block:: yaml

    reviewer: mailto:user@example.com

    labels:
      include:
        json-schema: boolean
        question: Include?
        required: true

    flows:
      label:
        steps:
          - uses: github:insilica/srvc-pubmed-search
            query: angry bees

          - run-embedded: remove-reviewed

          - run-embedded: label-web
            labels: [include]
            port: 5005
