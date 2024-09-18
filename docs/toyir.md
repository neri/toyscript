# Format of ToyIR

* ToyIR is an intermediate language designed for internal processing of ToyScript. Not for external exchange.
* Most opcodes have corresponding WebAssembly instructions, but they differ significantly from raw WebAssembly in that they make stack inputs and outputs explicit.
* Variable length using an array of 32-bit words, with the first word in the same format in all opcodes.



