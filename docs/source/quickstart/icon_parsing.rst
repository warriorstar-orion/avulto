Icon Parsing and Extraction
===========================

Now let's move on to ways Avulto can operate on DMI files. A fairly common task
one might find themselves facing is searching a project's DMI files for an icon
with the name "computer". Rather than trying to do this manually, a simple Avulto
script can do the job for us:

.. code-block:: python

    from pathlib import Path
    from avulto import DMI

    for iconfile in Path("icons").glob("**/*.dmi"):
        dmi = DMI.from_file(iconfile)
        if 'computer' in dmi.state_names():
            print(iconfile)

That's all that's necessary to quickly search the entire project for icons with
a given state name.

We can also export image data from icon files, which we will examine later.
