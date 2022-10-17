window.SIDEBAR_ITEMS = {"struct":[["GateSpec","Specifies a gate within a circuit. A gate will have a type, will refer to a row of variables, and will have associated vector of coefficients."],["ShiftedScalar","A variable that corresponds to scalar that is shifted by a certain amount."],["System","A set of gates within the circuit. It carries the index for the next available variable, and the vector of [`GateSpec`] created so far. It also keeps track of the queue of generic gates and cached constants."],["Var","A variable in our circuit. Variables are assigned with an index to differentiate from each other. Optionally, they can eventually take as value a field element."],["WitnessGenerator","Carries a vector of rows corresponding to the witness, a queue of generic gates, and stores the cached constants"]],"trait":[["Cs","This trait includes all the operations that can be executed by the elements in the circuits. It allows for different behaviours depending on the struct for which it is implemented for. In particular, the circuit mode and the witness generation mode."]]};