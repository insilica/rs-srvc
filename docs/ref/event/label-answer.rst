==================
Label Answer Event
==================

A label answer event contains an answer to a :doc:`label <label>` for a particular event.
Most label answers reference a :doc:`document event <document>`, but some reference other :doc:`answers <label-answer>` or even :doc:`labels <label>`.

Label answer events follow this format:

.. code-block:: json

    {
      "data": {
        "answer": true,
        "event": "QmZp5xnczbBDvAd2ma88Q2bRFkJiKeqxQt9iN6DHc527iR",
        "label": "QmYqmthq6E7aRyGgPmDZpWtL3Lk6UqM2RmCWLC1oVbmaxF",
        "reviewer": "mailto:user@example.com",
        "timestamp": 1673396981
      },
      "hash": "QmQqSvTuegWRQWGXbWwgtzKtD71fGhvoiUWwSsau971gTc",
      "type": "label-answer"
    }

All properties are required.

``answer`` may be any JSON value that is allowed by the label definition.
In this example, answer is a boolean value.
If the :doc:`label <label>` has a ``json-schema`` property, ``answer`` must be valid according to the schema.

``event`` is the hash of the event that the answer corresponds to.

``label`` is the hash of the label that is being answered.

``reviewer`` is the URI of the reviewer who created the answer.

``timestamp`` is a number representing the `Unix time <https://en.wikipedia.org/wiki/Unix_time>`_ when the answer was created.

History
=======

Version 0.18.0_ (2023-05-04)
---------------------------

.. _0.18.0: https://github.com/insilica/rs-srvc/releases/tag/v0.18.0

- ``document`` was renamed to ``event``. ``label-answers`` may now reference any type of event, not only documents.

