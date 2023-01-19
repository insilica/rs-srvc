==================
pubmed-search step
==================

Documents are imported from PubMed_ using the PubMed search step_.
The step follows this format:

.. _pubmed: https://pubmed.ncbi.nlm.nih.gov/
.. _step: https://github.com/insilica/srvc-pubmed-search/

.. code-block:: yaml

      - uses: github:insilica/srvc-pubmed-search
        query: "your search terms"

The step requires a ``query`` property to specify the search terms.
All documents that match the query will be imported.
For example, this flow uses a `query <https://pubmed.ncbi.nlm.nih.gov/?term=angry+bees>`_ that matches PubMed articles containing the words "angry bees":

.. code-block:: yaml

    reviewer: mailto:user@example.com

    flows:
      import-pubmed:
        steps:
          - uses: github:insilica/srvc-pubmed-search
            query: angry bees

Advanced queries
================

Advanced queries can be created in the `PubMed Advanced Search Builder <https://pubmed.ncbi.nlm.nih.gov/advanced/>`_.
Build a query in the Builder, and copy the text of the "Query box" into the step's query property.
A `query <https://pubmed.ncbi.nlm.nih.gov/?term=(angry+bees)+AND+(brain)>`_ that matches articles with both "angry bees" and "brain" looks like this:

.. code-block:: yaml

    reviewer: mailto:user@example.com

    flows:
      import-pubmed:
        steps:
          - uses: github:insilica/srvc-pubmed-search
            query: (angry bees) AND (brain)
