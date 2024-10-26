# Paths

`avulto.Path` wraps type paths and provides `parent_of(other, strict=False)` and
`child_of(other, strict=False)` for comparing paths. When `strict`, the current
path does not count as a parent or child of itself. It also supports the
division operator for easy path suffixing, and other properties for getting
different parts of the path.

```py
from avulto import Path as p

>>> p('/obj/machinery').parent_of('/obj/machinery/oven')
True

>>> p('/turf').child_of('/turf', strict=True)
False

>>> p('/obj/machinery') / 'microwave'
/obj/machinery/microwave

>>> p('/obj/machinery/microwave').parent
/obj/machinery

>>> p('/obj/machinery/microwave').stem
"microwave"
```
