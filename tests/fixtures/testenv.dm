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
