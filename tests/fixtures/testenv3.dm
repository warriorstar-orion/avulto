/obj/test_object

/obj/test_object/proc/var_and_return()
	var/local1 = 3
	var/local2 = local1
	return list(/obj{name="foo"} = 3, /obj/test_object = list("a" = 3, "b" = 4))
