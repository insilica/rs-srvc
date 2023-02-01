=============
recogito step
=============

The recogito step provides a web interface for a reviewer to assign annotations to documents.

The step follows this format:

.. code-block:: yaml

      - run-embedded: html https://raw.githubusercontent.com/sysrev/srvc-hello/main/src/resources/public/recogito.html
        labels: [annotation]
        port: 5006

The ``labels`` property defines which label to assign annotation answers to.
Only the first label is used.
Any others are ignored.

The ``port`` property sets the port number for the web server to listen on.
It is optional.
If omitted, the server will listen on an arbitrary free port.

Example ``sr.yaml``:

.. code-block:: yaml

    reviewer: mailto:user@example.com

    labels:
      annotation:
        question: Annotation

    flows:
      recogito:
        steps:
          - uses: github:insilica/srvc-pubmed-search
            query: angry bees

          - run-embedded: remove-reviewed

          - run-embedded: html https://raw.githubusercontent.com/sysrev/srvc-hello/main/src/resources/public/recogito.html
            labels: [annotation]
            port: 5006
