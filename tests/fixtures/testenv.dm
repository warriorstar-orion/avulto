/obj/foo
	icon = 'icon1.dmi'
	icon_state = "red_circle"
	var/a = 3

/obj/foo/proc/proc1(mob/M)
	var/m = M
	proc2(m)
	return m

/obj/foo/proc/proc2(mob/M)
	return

/obj/foo/bar
	a = 4

/obj/foo/baz

/obj/test_object

/obj/test_object/proc/var_and_return()
	var/local1 = 3
	var/local2 = local1
	return list(/obj{name="foo"} = 3, /obj/test_object = list("a" = 3, "b" = 4))


/obj/test_object/proc/example_call()

/obj/test_object/proc/test_visit_call()
	if(!example_call())
		example_call(var_anr_return())


/obj/test_object_2

/obj/test_object_2/proc/dupe_named_proc()
	return

/obj/test_object_2/proc/dupe_named_proc()
	return
