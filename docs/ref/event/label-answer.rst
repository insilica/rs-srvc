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
        "label": "QmdfYCe3UZ1xD39yj1w34EnkqsJtenPjeug7urWMkpUtei",
        "reviewer": "mailto:user@example.com",
        "timestamp": 1673396981
      },
      "hash":"QmeHHhQu1FFe2T9NHoxpsQmJoeW5aAi8T77HPSJVG47YvD",
      "type":"label-answer"
    }

All properties are required.

``answer`` may be any JSON value that is allowed by the label definition.
In this example, answer is a boolean value.

``document`` is the hash of document that the answer belongs to.

``label`` is the hash of the label that is being answered.

``reviewer`` is the URI of the reviewer who created the answer.

``timestamp`` is a number representing the `Unix time <https://en.wikipedia.org/wiki/Unix_time>`_ when the answer was created.
