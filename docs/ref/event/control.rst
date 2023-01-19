=============
Control Event
=============

A control event is used when a step needs to communicate with the SRVC software.
It is not saved to sink.jsonl unless sink-all-events is set to true.

There is currently only one kind of control event.
When a step runs a web server, it is used to inform SRVC of the port that the web server is listening on.

Control events follow this format:

.. code-block:: json

    {
      "data": {
        "http-port": 31157,
        "timestamp": 1673392440
      },
      "hash":"Qma84opq86nmbXTB5Lgof1wgYLXA9RQe4n9BidmmmyMs4x",
      "type": "control"
    }
