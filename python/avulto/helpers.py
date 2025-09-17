import random

from avulto import Dmlist, DME

def fill_with_ones(l: Dmlist) -> dict:
    """
    Returns a dictionary version of the passed in Dmlist with all unset values set to 1.
    """
    if not isinstance(l, Dmlist):
        return l

    final_list = {}

    for key in l.keys():
        if l[key]:
            final_list[key] = l[key]
        else:
            final_list[key] = 1

    return final_list


def pickweight(l: dict):
    """
    Returns a weighted pick from the items in l.
    """
    total = 0
    item = None
    for item, val in l.items():
        total += val

    total = random.uniform(0, total)
    for item, val in l.items():
        total -= val
        if total <= 0:
            return item

    return None


def pick_weight_recursive(list_to_pick):
    result = pickweight(fill_with_ones(list_to_pick))
    while isinstance(result, Dmlist):
        result = pickweight(fill_with_ones(result))
    return result

def prob(p):
    r = random.randint(1, 100)
    if r <= p:
        return 1
    return 0
