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
        "json-schema": {
          "$schema": "http://json-schema.org/draft-07/schema",
          "$id": "https://docs.sysrev.com/schema/label-answer/boolean-v2.json",
          "title": "Boolean answer",
          "description": "A boolean label answer",
          "type": "boolean"
        },
        "question": "Include?",
        "required": false
      },
      "hash":"QmYqmthq6E7aRyGgPmDZpWtL3Lk6UqM2RmCWLC1oVbmaxF",
      "type":"label"
    }

``data.id`` is required.
It is a string that allows SRVC to identify past versions of the same label.

``data.json-schema`` is optional.
It is a `JSON Schema <https://json-schema.org/>`_ object.
If present, :doc:`label answers <label-answer>` will be validated against the schema.

``data.question`` is required.
It is a string that is typically shown to the reviewer as a prompt.

``data.required`` is required.
It is a boolean that specifies whether a label must be answered for a document, or if the answer may be left blank.
