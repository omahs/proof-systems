initSidebarItems({"enum":[["CurrOrNext","A row accessible from a given row, corresponds to the fact that we open all polynomials at `zeta` and `omega * zeta`."],["GateLookupTable","Enumerates the different ‘fixed’ lookup tables used by individual gates"],["GateType","The different types of gates the system supports. Note that all the gates are mutually exclusive: they cannot be used at the same time on single row. If we were ever to support this feature, we would have to make sure not to re-use powers of alpha across constraints."],["LookupsUsed","Specifies whether a constraint system uses joint lookups. Used to make sure we squeeze the challenge `joint_combiner` when needed, and not when not needed."]],"fn":[["combine_table_entry","Let’s say we want to do a lookup in a “vector-valued” table `T: Vec<[F; n]>` (here I am using `[F; n]` to model a vector of length `n`)."],["get_table",""]],"struct":[["CircuitGate",""],["GatesLookupMaps","Specifies mapping from positions defined relative to gates into lookup data."],["GatesLookupSpec","Specifies the relative position of gates and the fixed lookup table (if applicable) that a given lookup configuration should apply to."],["JointLookup","A spec for checking that the given vector belongs to a vector-valued lookup table."],["LocalPosition","A position in the circuit relative to a given row."],["LookupInfo","Describes the desired lookup configuration."],["SingleLookup","Look up a single value in a lookup table. The value may be computed as a linear combination of locally-accessible cells."]],"type":[["LookupTable",""]]});