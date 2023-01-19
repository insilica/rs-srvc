===========
Label Event
===========

A label event contains a label definition.
The definition is used by labelling steps to assign an :doc:`answer <label-answer>` to a :doc:`document <document>`.

Label events follow this format:

.. code-block:: json

    {
      "data": {
        "id": "include",
        "question": "Include?",
        "required": false,
        "type": "boolean"
      },
      "hash":"QmdfYCe3UZ1xD39yj1w34EnkqsJtenPjeug7urWMkpUtei",
      "type":"label"
    }

``data.id`` is required.
It is a string that allows SRVC to identify past versions of the same label.

``data.question`` is required.
It is a string that is typically shown to the reviewer as a prompt.

``data.required`` is required.
It is a boolean that specifies whether a label must be answered for a document, or if the answer may be left blank.

``data.type`` is required.
It is a string that may be used by labelling steps to determine what form label answers may take.
