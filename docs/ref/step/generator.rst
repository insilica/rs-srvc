==============
generator step
==============

The generator step imports :doc:`events </ref/event/index>` from a file or URL.
Often the events imported will be :doc:`documents </ref/event/document>`, but they may also be :doc:`labels </ref/event/label>` and :doc:`label answers </ref/event/label-answer>`.
The step follows this format:

.. code-block:: yaml

    - run-embedded: generator "file or URL"

Example using docs.jsonl file
=============================

Suppose that we create a docs.jsonl file containing several documents:

.. code-block:: json

    {"data":{"title":"The Beehive Theory"},"type":"document","uri":"https://pubmed.ncbi.nlm.nih.gov/16999303/"}
    {"data":{"title":"Do ambient urban odors evoke basic emotions?"},"type":"document","uri":"https://pubmed.ncbi.nlm.nih.gov/24860522/"}
    {"data":{"title":"All abuzz. Angry bees ignite an unexpected MCI."},"type":"document","uri":"https://pubmed.ncbi.nlm.nih.gov/16999303/"}

We can create a flow that imports the documents like this:

.. code-block:: yaml
    :caption: ``sr.yaml``

    reviewer: mailto:user@example.com

    flows:
      import-docs:
        steps:
          - run-embedded: generator docs.jsonl

Example using a URL
===================

We can import events from a URL instead of a file.
This example imports all of the documents, labels, and label answers from the public Mangiferin_ systematic review.

.. _mangiferin: https://sysrev.com/o/2/p/21696

.. code-block:: yaml
    :caption: ``sr.yaml``

    reviewer: mailto:user@example.com

    flows:
      import-mangiferin:
        steps:
          - run-embedded: generator https://sysrev.com/web-api/srvc-events?project-id=21696

History
=======

Version 0.9.0_ (2022-12-12)
---------------------------

.. _0.9.0: https://github.com/insilica/rs-srvc/releases/tag/v0.9.0

- ``generator-file`` was renamed to ``generator``, and URL support was added. ``generator-file`` is now an alias for ``generator``.
