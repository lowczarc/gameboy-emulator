# GAMEBOY CPU INSTRUCTION/OPCODE

16bits numbers are stored as little endian.

At first glance it seems that opcodes are sometimes following a pattern:
```
0b aa bbb c ddd
```

WHERE aa =>
	Consistently 1x for arithmetic/logic operators (except INC & DEC):
		-> 10 for AND ONLY FOR register operations with A
		-> 11 for operations between A and a parameter (though other things like JP or some LD operations uses this)
	LD between 2 8bits register seems to be the only one that uses 01 as aa EXCEPT HALT which seems to be triggered by the same opcode as a theoritical (arguably useless) `LD (HL) (HL)`
	I cannot find a pattern for 00 and 11 :/

If opcode are taking registers as argument, bbb and ddd can be set to the register's number:
```
	B:		000
	C:		001
	D:		010
	E:		011
	H:		100
	L:		101
	(HL):	110
	A: 		111
```

If 16bits registers are used only the 2 most significant bits of bbb and ddd are arguments. The register numbers become:
```
	BC:		00
	DE:		01
	HL:		10
	SP:		11
```
Note that except for SP that isn't composed of 8bits registers, their number is just their equivalent 8bit registers>>1.
