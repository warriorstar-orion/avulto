// /obj/foo
// 	var/test_var = 3

// /obj/foo/proc/var_assignments()
// 	var/local_num = 3
// 	var/local_var = local_num

// /obj/foo/proc/var_lists()
	
// 	var/list/my_second_list = list("foo" = "bar", "baz" = 5)
// 	var/list/my_list = list("foo", my_second_list, 3)

// /obj/foo/proc/method1()
// 	return

// /obj/foo/proc/method2()
// 	return

// /obj/foo/proc/if_statements()
// 	if(test_var == 3)
// 		method1()
// 	else
// 		method2()

/obj/test_object

/obj/test_object/proc/var_and_return()
	var/local1 = 3
	var/local2 = local1
	return local2
