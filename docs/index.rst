==================
SRVC
==================

.. rubric:: Everything you need to use SRVC (SysRev Version Control).

SRVC is an open-source document review system that provides full privacy and control over your data.
It is versatile, and can be extended with Python, R, or any other language.
The data model is flexible, and supports managing data with `git <https://git-scm.com/>`_.

.. _index-first-steps:

First steps
===========

Are you new to SRVC? This is the place to start!

* :doc:`Installation <installation>`
* :doc:`Overview <overview>`
* :doc:`Reference <ref/index>`

Quickstart
==========
First, :doc:`Install SRVC <installation>`, then clone `srvc-template <https://github.com/insilica/srvc-template>`_  and :

.. code-block:: console

   $ git clone https://github.com/insilica/sfac.git
   $ cd srvc-template
   $ sr flow pubmed-search # Pull "angry bees" from pubmed
   $ sr flow label # Start labeling interface on port 5005

Now you can alter ``sr.yaml`` to modify the pubmed query or begin building your own SRVC projects.
