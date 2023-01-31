==================
Label Answer Event
==================

A label answer event contains an answer to a :doc:`label <label>` for a particular :doc:`document <document>`.

Label answer events follow this format:

.. code-block:: json

    {
      "data": {
        "answer": true,
        "document": "QmZp5xnczbBDvAd2ma88Q2bRFkJiKeqxQt9iN6DHc527iR",
        "label": "QmYqmthq6E7aRyGgPmDZpWtL3Lk6UqM2RmCWLC1oVbmaxF",
        "reviewer": "mailto:user@example.com",
        "timestamp": 1673396981
      },
      "hash": "QmZPAta9rGPJkHbfZSacCP4C1gqjRudxt8p7xNpr1P7NTQ",
      "type": "label-answer"
    }

All properties are required.

``answer`` may be any JSON value that is allowed by the label definition.
In this example, answer is a boolean value.
If the :doc:`label <label>` has a ``json-schema`` property, ``answer`` must be valid according to the schema.

``document`` is the hash of document that the answer belongs to.

``label`` is the hash of the label that is being answered.

``reviewer`` is the URI of the reviewer who created the answer.

``timestamp`` is a number representing the `Unix time <https://en.wikipedia.org/wiki/Unix_time>`_ when the answer was created.
