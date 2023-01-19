=============
Overview
=============

SRVC is an open-source document review system.
It is designed to make common review tasks fast and easy.
Here is an overview of how to create a review project with SRVC.

Create a project
================

Every SRVC project is defined by an sr.yaml file.
It specifies the project configuration, labels, and review flows.

Here is a sample ``sr.yaml`` from the `srvc-template repository <https://github.com/insilica/srvc-template/blob/main/sr.yaml>`_.

.. code-block:: yaml

    reviewer: mailto:user@example.com

    labels:
      category:
        question: Category
        type: categorical
        categories:
          - A
          - B
          - C

      include:
        question: Include?
        required: true
        type: boolean

    flows:
      pubmed-search:
        steps:
          - uses: github:insilica/srvc-pubmed-search
            query: angry bees

      label:
        steps:
          - run-embedded: generator sink.jsonl

          - run-embedded: remove-reviewed

          - run-embedded: label-web
            labels: [include, category]
            port: 5005

To follow along, create a directory for your new project, and place this example ``sr.yaml`` file in that directory.

Note: This example requires `Nix <https://nixos.org>`_ to be installed.

Import documents
================

With the ``sr.yaml`` file given above, documents can be imported from PubMed with the command:

.. code-block:: shell

    sr review pubmed-search

As the :doc:`search step </ref/step/pubmed-search>` runs, you can see :doc:`documents </ref/event/document>` being appended to a file named ``sink.jsonl``.

This step only needs to be run once, but can be repeated without harm.
Any new or changed documents will simply be added to the existing documents.

Review documents
================

Reviewing documents involves assigning answers for the "include" and "category" labels.
Get started with the command:

.. code-block:: shell

    sr review label

You should see the text ``Listening on http://127.0.0.1:5005``.
Visiting that address in a browser will load an interface that shows a document and allows submitting answers for that document.
After you submit an answer, you should see that it has been appended to ``sink.jsonl``.

Collaborate
=============

One way to collaborate with other reviewers is to use a git repository for your SRVC project directory.
Reviewers can merge their work by merging their sink files.
If there are any git conflicts, simply accept all additions from both reviewers.
The SRVC data model allows for seamless merging.
