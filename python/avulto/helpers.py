import random

from avulto import Dmlist

def fill_with_ones(list_to_pad):
    if not isinstance(list_to_pad, Dmlist):
        return list_to_pad

    final_list = dict()

    for key in list_to_pad.keys():
        if list_to_pad[key]:
            final_list[key] = list_to_pad[key]
        else:
            final_list[key] = 1

    return final_list


def pickweight(L):
    total = 0
    item = None
    for item, val in L.items():
        total += val

    total = random.uniform(0, total)
    for item, val in L.items():
        total -= val
        if total <= 0:
            return item

    return None


def pick_weight_recursive(list_to_pick):
    result = pickweight(fill_with_ones(list_to_pick))
    while isinstance(result, Dmlist):
        result = pickweight(fill_with_ones(result))
    return result

def prob(P):
    r = random.randint(1, 100)
    if r <= P:
        return 1
    return 0
