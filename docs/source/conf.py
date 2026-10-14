# Configuration file for the Sphinx documentation builder.
#
# For the full list of built-in configuration values, see the documentation:
# https://www.sphinx-doc.org/en/master/usage/configuration.html

# -- Project information -----------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#project-information

project = 'avulto'
copyright = '2026 Warriorstar Orion'
author = 'Warriorstar Orion'
release = 'v0.5.0'
version = release

extensions = [
    'sphinx.ext.autodoc',
    'sphinx.ext.napoleon',
    'sphinx.ext.autosummary',
    'sphinx.ext.intersphinx',
    'sphinxext.opengraph',
    'autoapi.extension',
]

autoapi_dirs = ["../../python/avulto"]
autoapi_generate_api_docs = False
autoapi_file_patterns = ["*.pyi", "*.py"]
autoapi_ignore = ["*/ast/*"]

autosummary_generate = True
autosummary_imported_members = True

default_role = "py:obj"

intersphinx_mapping = {
    'python': ('https://docs.python.org/3', None),
    'Pillow': ('https://pillow.readthedocs.io/en/stable/', None),
}

templates_path = ['_templates']
exclude_patterns = []

pygments_style = 'lightbulb'

html_theme = 'furo'
html_static_path = ['_static']

ogp_site_url = "https://warriorstar-orion.github.io/avulto/"
