`DME` --- Code Reflection
=========================

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
   .. autoapiproperty:: avulto.TypeDef.vars

      .. autoapimethod:: avulto.VarHolder.names
      .. autoapimethod:: avulto.VarHolder.all
      .. autoapimethod:: avulto.VarHolder.declared
      .. autoapimethod:: avulto.VarHolder.modified
      .. autoapimethod:: avulto.VarHolder.unmodified

   .. autoapiproperty:: avulto.TypeDef.procs