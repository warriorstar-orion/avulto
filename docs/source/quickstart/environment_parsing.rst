Environment Parsing
===================

Avulto isn't limited to working with map and icon files; it has basic support
for inspecting the codebase itself.

Let's say you are a typical SS13 codebase, and you have a Syndicate antagonist
with an uplink, and it has many uplink items, like so:

.. code-block:: c

	/datum/uplink_item/stealthy_weapons/garrote
		name = "Fiber Wire Garrote"
		item = /obj/item/garrote
		reference = "GAR"
		cost = 30


You want to get a listing of every uplink item, and how much it costs, in one
list. We can get this list like so:

.. code-block:: python

	from pathlib import Path
	from collections import namedtuple
	from avulto import DME

	dme = DME.from_file("paradise.dme")

	for pth in dme.typesof('/datum/uplink_item'):
		typedecl = dme.type_decl(pth)
		print(f"Name: {typedecl.value('name')} Cost: {typedecl.value('cost')}")


And the result will end up looking something like this:

.. code-block::

	Name: Carbine - 40mm Grenade Ammo Box Cost: 20
	Name: Stechkin APS - 10mm Magazine Cost: 10
	Name: Stechkin APS - 10mm Armour Piercing Magazine Cost: 15
	Name: Stechkin APS - 10mm Incendiary Magazine Cost: 15
	Name: Stechkin APS - 10mm Hollow Point Magazine Cost: 20
	Name: Box of Bioterror Syringes Cost: 25
	Name: Bulldog - 12g Buckshot Magazine Cost: 10
	Name: Bulldog - 12g XL Magazine Duffel Bag Cost: 60
	Name: Bulldog - 12g Ammo Duffel Bag Cost: 60
	Name: Bulldog - 12g Dragon's Breath Magazine Cost: 10
	...
