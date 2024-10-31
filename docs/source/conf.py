# Configuration file for the Sphinx documentation builder.
#
# For the full list of built-in configuration values, see the documentation:
# https://www.sphinx-doc.org/en/master/usage/configuration.html

# -- Project information -----------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#project-information

project = 'avulto'
copyright = '2024 Warriorstar Orion'
author = 'Warriorstar Orion'
release = 'v0.0.15'
version = release


extensions = [
    'sphinx.ext.autodoc',
    'sphinx.ext.autosummary',
    'sphinx.ext.intersphinx',
    # 'sphinx.ext.todo',
    # 'sphinx.ext.inheritance_diagram',
    # 'sphinx.ext.autosectionlabel',
    # 'sphinx.ext.napoleon',
    # 'sphinx_rtd_theme',
]

autosummary_generate = True
autosummary_imported_members = True

intersphinx_mapping = {
    'python': ('https://docs.python.org/3', None),
    'Pillow': ('https://pillow.readthedocs.io/en/stable/', None),
}

templates_path = ['_templates']
exclude_patterns = []

# -- Options for HTML output -------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#options-for-html-output

# html_theme = 'sphinx_rtd_theme'
html_theme = 'alabaster'
# html_static_path = ['_static']
# html_css_files = [
#     'custom.css',
# ]
# html_permalinks = False
# html_permalinks_icon = "BLAH"
# html_add_permalinks = "X"
# html_permalinks_icon = 'X'

# html_sidebars = {
#     # '**': [
#     #     'index.html',
#     #     # 'navigation.html',
#     #     # 'relations.html',
#     #     # 'searchbox.html',
#     #     # 'donate.html',
#     # ]
# }

html_sidebars = {
   '**': ['globaltoc.html', 'sourcelink.html', 'searchbox.html'],
}

# html_theme_options = {
#     "body_text": "#ccc",
#     "link": "#eee",
# }
