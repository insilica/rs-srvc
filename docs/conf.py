# Configuration file for the Sphinx documentation builder.
#
# For the full list of built-in configuration values, see the documentation:
# https://www.sphinx-doc.org/en/master/usage/configuration.html

import subprocess

commit_id = subprocess.check_output(['git', 'rev-parse', 'HEAD']).strip().decode('ascii')

# -- Project information -----------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#project-information

project = 'SRVC'
copyright = '2023, Insilica'
author = 'Insilica'

# -- General configuration ---------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#general-configuration

extensions = []

templates_path = ['_templates']
exclude_patterns = ['_build', 'Thumbs.db', '.DS_Store']

# The root toctree document.
root_doc = "contents"

# -- Options for HTML output -------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#options-for-html-output

html_favicon = 'favicon.ico'
html_show_copyright = False
html_show_sphinx = False
html_static_path = ['_static']
html_theme = 'sphinx_rtd_theme'
html_context = {
  'display_github': True,
  'github_user': 'insilica',
  'github_repo': 'rs-srvc',
  'github_version': commit_id + '/docs/',
}
