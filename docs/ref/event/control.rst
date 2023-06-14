=============
Control Event
=============

A control event is used when a step needs to communicate with the SRVC software.
It is not saved to sink.jsonl unless sink-control-events is set to true.

There is currently only one kind of control event.
When a step runs a web server, it is used to inform SRVC of the port that the web server is listening on.

Control events follow this format:

.. code-block:: json

    {
      "data": {
        "http-port": 31157,
        "timestamp": 1673392440
      },
      "hash": "Qma84opq86nmbXTB5Lgof1wgYLXA9RQe4n9BidmmmyMs4x",
      "type": "control"
    }

History
=======

Version 0.20.0_
---------------------------

.. _0.20.0: https://github.com/insilica/rs-srvc/releases/tag/v0.20.0

- ``sink-all-events`` was renamed to ``sink-control-events``.


