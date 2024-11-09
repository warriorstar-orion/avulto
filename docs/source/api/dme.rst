:class:`DME` --- Code Reflection
================================

.. class:: DME

   The :class:`DME` class provides reflection information for a BYOND project. This
   includes looking up available types, proc and var names on those types, and
   basic support for AST walking.

   :class:`DME` instances are created with the following methods:

   .. staticmethod:: from_file(filename: str | os.PathLike[str], parse_procs=False) -> DME

      Read the BYOND environment from the *filename* referring to a ".dme" file.
      If the optional *parse_procs* argument is :const:`True`, reflection data is
      made available for all procs.

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

   .. method:: walk_proc(path: Path | str, proc: str, walker)

      Performs an AST walk of the proc *proc* on the object specified by *path*.
      The argument *walker* is expected to be a Python object which exposes
      methods for each kind of AST node you wish to visit. Each method should
      take an argument *node*, which will be filled in with information about
      that node in the AST. The currently available visitors are:

      - ``visit_AssignOp``
      - ``visit_BinaryOp``
      - ``visit_Break``
      - ``visit_Call``
      - ``visit_Constant``
      - ``visit_Continue``
      - ``visit_Crash``
      - ``visit_Del``
      - ``visit_DoWhile``
      - ``visit_Expr``
      - ``visit_For``
      - ``visit_ForList``
      - ``visit_ForRange``
      - ``visit_Goto``
      - ``visit_If``
      - ``visit_InterpString``
      - ``visit_Label``
      - ``visit_New``
      - ``visit_ParentCall``
      - ``visit_Prefab``
      - ``visit_Resource``
      - ``visit_Return``
      - ``visit_SelfCall``
      - ``visit_Setting``
      - ``visit_Spawn``
      - ``visit_Switch``
      - ``visit_TernaryOp``
      - ``visit_Throw``
      - ``visit_TryCatch``
      - ``visit_Var``
      - ``visit_While``

      As with :class:`ast.NodeVisitor`, child nodes of a custom visitor method
      will not be visited. There is currently no analogous ``generic_visit``
      support.

.. class:: TypeDecl

   The :class:`TypeDecl` class returns basic information about a type declared
   in the :class:`DME` file.

   .. method:: proc_names() -> list[str]

      Returns a list of proc names for the type declaration.

   .. method:: var_names() -> list[str]

      Returns a list of variables names for the type declaration. This does not
      include variables declared in the type's parents.

   .. method:: value(name: str)

      Returns a Python representation of the variable *name*. This will lookup
      values of variables declared in the type's parents.

   .. method:: walk_proc(proc: str, walker)

      Use the AST *walker* to walk the proc named *proc*. A convenience method
      for :py:meth:`DME.walk_proc`.
