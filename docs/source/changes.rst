Important Changes
#################

v0.1.0
======

The upgrade from v0.0.26 to v0.1.0 comes with several important and/or
backwards-incompatible API changes.

Name Changes
************

DME#typedecl is now DME#type_decl.

Proc Walking
************

1. Proc walking is no longer accessible from DME. Instead, one must have a valid
   ProcDecl, and call ProcDecl#walk.

2. When walking DmLists, all keys and values are also expressions, and can be
   walked independently.
