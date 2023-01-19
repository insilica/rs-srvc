==============
Document Event
==============

A document event contains an article or other data that is to be reviewed.

Document events follow this format:

.. code-block:: json

    {
      "data": {"title": "The Beehive Theory"},
      "hash":"QmZp5xnczbBDvAd2ma88Q2bRFkJiKeqxQt9iN6DHc527iR",
      "type":"document",
      "uri":"https://pubmed.ncbi.nlm.nih.gov/16999303/"
    }

The data property may be any JSON object.
Labelling and review steps may only understand certain forms of data, and may assign special meaning to certain keys.
Common keys include title and abstract.

The uri property is optional, but it is strongly recommended to assign it whenever a URI_ exists.
Steps may provide a link to reviewers based on the uri.
The uri also helps SRVC to deduplicate documents when there is more than one version of a document's data.

.. _uri: https://en.wikipedia.org/wiki/Uniform_Resource_Identifier
