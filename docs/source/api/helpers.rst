Helpers
=======

Avulto provides several Python functions that can be useful for inspecting
codebases.

.. function:: fill_with_ones(l: Dmlist) -> dict

    Returns a dictionary version of the passed in list *l* with all unset values set to 1.

.. function:: pickweight(d: dict) -> any

    Returns a weighted pick from the items in *d*.

.. function:: pick_weight_recursive(l: Dmlist) -> any

    Returns a recursive weighted pick from the items in *l*.

.. function:: prob(p: int) -> int

    Returns 1 with probability *p* percent, otherwise 0. Equivalent to BYOND's prob_.

    .. _prob: https://www.byond.com/docs/ref/#/proc/prob