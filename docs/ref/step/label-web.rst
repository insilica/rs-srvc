==============
label-web step
==============

The label-web step provides a web interface for a reviewer to assign answers to documents.
The step follows this format:

.. code-block:: yaml

      - run-embedded: label-web
        labels: [include, category]
        port: 5005

The ``labels`` property defines which labels to show in the interface.

The ``port`` property sets the port number for the web server to listen on.
It is optional.
If omitted, the server will listen on an arbitrary free port.

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
          - run-embedded: remove-reviewed

          - run-embedded: label-web
            labels: [include]
            port: 5005

    sources:
      - step:
          uses: github:insilica/srvc-pubmed-search
          query: angry bees
