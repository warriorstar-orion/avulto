``DME`` --- Code Reflection
===========================

.. autoapiclass:: avulto.DME

   The :class:`DME` class provides reflection information for a BYOND project.
   This includes looking up available types, proc and var names on those types,
   and basic support for AST walking.

   :class:`DME` instances are created with the following methods:

   .. autoapimethod:: avulto.DME.from_file

   Once instantiated, the following properties and methods are available:

   .. autoapiproperty:: avulto.DME.types

   .. autoapimethod:: avulto.DME.typesof
   .. autoapimethod:: avulto.DME.subtypesof

.. autoapiclass:: avulto.ProcDef

   .. autoapimethod:: avulto.ProcDef.walk

      There is currently no analogous ``generic_visit`` support.

.. autoapiclass:: avulto.VarDef

   .. autoapiproperty:: avulto.VarDef.name
   .. autoapiproperty:: avulto.VarDef.type_path
   .. autoapiproperty:: avulto.VarDef.declared_type
   .. autoapiproperty:: avulto.VarDef.const_val
   .. autoapiproperty:: avulto.VarDef.source_loc

.. autoapiclass:: avulto.TypeDef

   .. autoapiproperty:: avulto.TypeDef.path
   .. autoapiproperty:: avulto.TypeDef.source_loc
   .. autoapiproperty:: avulto.TypeDef.procs      
   .. autoapiproperty:: avulto.TypeDef.vars

.. autoapiclass:: avulto.ProcHolder

   A collection of the procs on a `TypeDef`, returned from `TypeDef.procs`.

   .. autoapimethod:: avulto.ProcHolder.__getitem__

      Usage: ``procs[name]``. Return an array of `ProcDef` instances for proc
      declaration with the given *name*. Raises :exc:`KeyError` if the type has
      no such procs.

   .. autoapimethod:: avulto.ProcHolder.names
   .. autoapimethod:: avulto.ProcHolder.all
   .. autoapimethod:: avulto.ProcHolder.declared
   .. autoapimethod:: avulto.ProcHolder.modified
   .. autoapimethod:: avulto.ProcHolder.unmodified

.. autoapiclass:: avulto.VarHolder

   A collection of the vars on a `TypeDef`, returned from `TypeDef.vars`.

   .. describe:: vars[name]

      Return the `VarDef` for the var *name*. Raises :exc:`KeyError` if the type
      has no such var.

   .. describe:: iter(vars)

      Return an iterator over every `VarDef` on the type.

   .. autoapimethod:: avulto.VarHolder.names
   .. autoapimethod:: avulto.VarHolder.all
   .. autoapimethod:: avulto.VarHolder.declared
   .. autoapimethod:: avulto.VarHolder.modified
   .. autoapimethod:: avulto.VarHolder.unmodified         