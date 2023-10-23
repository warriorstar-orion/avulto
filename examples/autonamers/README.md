# Avulto Example: Autonamer Map Fixer

This directory contains a Python script used to demonstrate several aspects of
the Avulto library, namely DMM parsing, inspection, and modification, and
retrieving object definition data on the fly.

To use it, modify the variables `CODE_ROOT`, `MAPS`, and `dme` to point to your
SS13 codebase, the maps you wish to modify, and the name of the `.dme` file, and
then run `python autonamers.py`.

This example presumes you have the requisite mapping helpers in your codebase.

## How It Works

Autonamers work by being placed on top of airlocks. When map loading occurs,
they inspect the name of the `/area` they are on, and then change the name of
the airlock on the tile to the name of the `/area`. This allows airlocks to be
named something meaningful without having to make varedits to them in the DMM
file.

This script iterates over all tiles on a map, seeing if there exists an airlock
which has a varedited name, and if the name is the same as the area, removes the
varedit and places an autonamer on top of the airlock.