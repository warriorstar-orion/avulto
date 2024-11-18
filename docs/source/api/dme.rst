:class:`DME` --- Code Reflection
================================

.. class:: DME

   The :class:`DME` class provides reflection information for a BYOND project.
   This includes looking up available types, proc and var names on those types,
   and basic support for AST walking.

   :class:`DME` instances are created with the following methods:

   .. staticmethod:: from_file(filename: str | os.PathLike[str], parse_procs=False) -> DME

      Read the BYOND environment from the *filename* referring to a ".dme" file.
      If the optional *parse_procs* argument is :const:`True`, reflection data
      is made available for all procs.

      :raises: :class:`OSError`: If the file is not found or there was an error opening it.
      :raises: :class:`RuntimeError`: If there was an error parsing the DME environment.

   Once instantiated, the following methods are available:

   .. method:: typesof(prefix: Path | str) -> list[Path]

      Return a list of :class:`Path`\s which include the type *prefix* and any of
      its subtypes.

   .. method:: subtypesof(prefix: Path | str) -> list[Path]

      Return a list of :class:`Path`\s of any subtypes of *prefix*, excluding
      itself.

   .. method:: typedecl(path: Path | str) -> TypeDecl

      Returns the :class:`TypeDecl` of the type given by *path* if it exists.


.. class:: ProcDecl

   A declaration for a specific proc on a type. Note that a type may have
   multiple definitions of a given proc with the same name. This only represents
   one proc definition.

   .. method:: walk(walker)

      Use the AST *walker* to walk this proc.

      The argument *walker* is expected to be a Python object which exposes
      methods for each kind of AST node you wish to visit. Each method should
      take two arguments: *node*, which will be filled in with information about
      that node in the AST, and *source_info*, which includes lexical
      information about the AST node, such as line, column, and filename. The
      currently available visitors are:

      - ``visit_AssignOp``
      - ``visit_BinaryOp``
      - ``visit_Break``
      - ``visit_Call``
      - ``visit_Constant``
      - ``visit_Continue``
      - ``visit_Crash``
      - ``visit_Del``
      - ``visit_DoWhile``
      - ``visit_DynamicCall``
      - ``visit_Expr``
      - ``visit_ExternalCall``
      - ``visit_Field``
      - ``visit_ForInfinite``
      - ``visit_ForList``
      - ``visit_ForLoop``
      - ``visit_ForRange``
      - ``visit_Goto``
      - ``visit_Identifier``
      - ``visit_If``
      - ``visit_Index``
      - ``visit_Input``
      - ``visit_InterpString``
      - ``visit_Label``
      - ``visit_List``
      - ``visit_Locate``
      - ``visit_NewImplicit``
      - ``visit_NewMiniExpr``
      - ``visit_NewPrefab``
      - ``visit_ParentCall``
      - ``visit_Pick``
      - ``visit_ProcReference``
      - ``visit_Return``
      - ``visit_SelfCall``
      - ``visit_Setting``
      - ``visit_StaticField``
      - ``visit_Switch``
      - ``visit_TernaryOp``
      - ``visit_Throw``
      - ``visit_TryCatch``
      - ``visit_UnaryOp``
      - ``visit_Var``
      - ``visit_While``

      As with :class:`ast.NodeVisitor`, child nodes of a custom visitor method
      will not be visited. There is currently no analogous ``generic_visit``
      support.


.. class:: VarDecl

   The :class:`VarDecl` class returns basic information about a variable
   declared on a :class:`TypeDecl` type declaration.

   .. property:: name
      :type: str

      The name of the variable.

   .. property:: declared_type
      :type: Path | None

      The type of the variable, if available and declared.

   .. property:: const_val
      :type: any | None

      The initial value of the variable, if expressable as a constant.

.. class:: TypeDecl

   The :class:`TypeDecl` class returns basic information about a type declared
   in the :class:`DME` file.

   .. method:: proc_decls(name=None) -> list[ProcDecl]

      Returns a list of :class:`ProcDecl`\s for the type. If *name* is set, only
      proc declarations with that name will be returned.

   .. method:: proc_names() -> list[str]

      Returns a list of proc names for the type declaration.

   .. method:: var_decl(name: str, parents: bool=True) -> VarDecl

      Returns the variable declaration of the var *name*. If *parents* is
      :const:`True`, the type's parents will be checked for a variable
      declaration if not specified on the current type.

   .. method:: var_names() -> list[str]

      Returns a list of variables names for the type declaration. This does not
      include variables declared in the type's parents.
