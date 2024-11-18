# Configuration file for the Sphinx documentation builder.
#
# For the full list of built-in configuration values, see the documentation:
# https://www.sphinx-doc.org/en/master/usage/configuration.html

# -- Project information -----------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#project-information

project = 'avulto'
copyright = '2024 Warriorstar Orion'
author = 'Warriorstar Orion'
release = 'v0.1.2'
version = release

extensions = [
    'sphinx.ext.autodoc',
    'sphinx.ext.autosummary',
    'sphinx.ext.intersphinx',
    'sphinxext.opengraph',
]

autosummary_generate = True
autosummary_imported_members = True

intersphinx_mapping = {
    'python': ('https://docs.python.org/3', None),
    'Pillow': ('https://pillow.readthedocs.io/en/stable/', None),
}

templates_path = ['_templates']
exclude_patterns = []

html_theme = 'alabaster'
html_sidebars = {
   '**': ['globaltoc.html', 'sourcelink.html', 'searchbox.html'],
}

ogp_site_url = "https://warriorstar-orion.github.io/avulto/"
