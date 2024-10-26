# DME

The `DME` class allows parsing DM object code:

| Method                   | Description                                                                       |
| ------------------------ | --------------------------------------------------------------------------------- |
| `from_file(filename)`    | Creates a `DME` from the given `.dme` file.                                       |
| `typesof(prefix)`        | Returns a list of paths with the given `prefix`, including `prefix` if it exists. |
| `subtypesof(prefix)`     | Returns a list of paths with the given `prefix`, excluding `prefix`.              |
| `typedecl(path)`         | Returns a `TypeDecl` of the object `path`.                                        |

`TypeDecl` objects allow variable inspection:

| Method         | Description                                                |
| -------------- | ---------------------------------------------------------- |
| `proc_names()` | Returns a list of proc names for the type declaration.     |
| `var_names()`  | Returns a list of variable names for the type declaration. |
| `value(name)`  | Return a Python representation of the variable `name`.     |
