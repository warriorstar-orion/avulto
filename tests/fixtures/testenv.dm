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

/obj/node_parse

/obj/node_parse/proc/if_else_if_else(x)
	if(x == 1)
		return 1
	else if(x == 2)
		return 2
	else
		return 3

/obj/node_parse/proc/if_no_else(x)
	if(x)
		return 1

/obj/node_parse/proc/while_loop(x)
	while(x)
		x--
		x--

/obj/node_parse/proc/do_while_loop(x)
	do
		x--
	while(x)

/obj/node_parse/proc/for_loop()
	for(var/i = 0, i < 3, i++)
		return i

/obj/node_parse/proc/for_infinite()
	for()
		return

/obj/node_parse/proc/for_list(list/L)
	for(var/obj/O in L)
		return O

/obj/node_parse/proc/for_range()
	for(var/i in 1 to 10 step 2)
		return i

/obj/node_parse/proc/for_key_value(list/L)
	for(var/k, v in L)
		return v

/obj/node_parse/proc/switch_stmt(x)
	switch(x)
		if(1)
			return 1
		if(2 to 5)
			return 2
		else
			return 3

/obj/node_parse/proc/switch_no_default(x)
	switch(x)
		if(1, 2)
			return 1

/obj/node_parse/proc/try_catch()
	try
		throw EXCEPTION("oops")
	catch(var/exception/e)
		return e

/obj/node_parse/proc/vars_stmt()
	var/a = 1, b

/obj/node_parse/proc/setting()
	set name = "Setting"

/obj/node_parse/proc/del_stmt(x)
	del(x)

/obj/node_parse/proc/crash_stmt()
	CRASH("oops")
