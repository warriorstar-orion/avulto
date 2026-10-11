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

/obj/foo_2

/obj/base/proc/foobar()
	return

/obj/base/proc/barbaz()
	return

/obj/base/override/foobar()
	return

/obj/test_object

/obj/test_object/proc/var_and_return()
	var/local1 = 3
	var/local2 = local1
	make_test_call()
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

/proc/hell_yeah(foo)
	return foo

/datum/foo

/datum/foo/bar
	var/a = 1
	var/b = 2

/datum/foo/bar/New(a, b)
	src.a = a
	src.b = b

/datum/foo/baz

/obj/init_list_vardecls
	var/list/my_news = list(
		new /datum/foo/bar(3, 4),
		new /datum/foo/baz(10, 12),
	)