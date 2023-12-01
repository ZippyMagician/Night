# Night
**(WIP)** Interpreted concatenative stack-based language.

## About
TODO. Currently working on getting the major stuff implemented before I rewrite the readme and work on documentation.

- [x] Lexer
- [x] Basic interpreter
- [x] Operator structure
- [x] Builtin structure
- [x] Const symbol definitions
- [x] Blocks
- [x] Symbol definitions
- [x] Register definitions
- [x] Guard statements
- [ ] Arrays
- [x] Implement operators
- [ ] Implement builtins

Possible ideas
- Complex number support
- More math builtins
- Libraries
- Lower level operations

## Basic Syntax
```ruby
-> x| 4 7 +
x x * print

-> mults  (:a) :a ! ; 11 1 range { $a $I + } for
-> mults2 (:a) :a ! 9 { . $a + } loop
7 mults

-> dip (:dip) : :dip ! ; ? $dip
-> for (:for_f) {
	:for_f ! ; . len
	(:for_r :I) {
		. head : 1 drop :for_r ! ; :I ! ;
		$for_f ? $for_r
	} loop
}
```

## Some Definitions
May change once implemented.
| Function | Symbol | Def |
| -- | -- | -- |
| `pop` | `;` | Pop top value from stack |
| `dup` | `.` | Duplicate top value of stack |
| `defr` | `!` | Assign value to temp register |
| `undefr` | | Unassign value from temp register. Generally not specifically called |
| `join` | `,` | Joins top two values from stack into array |
| `I` | | Intermediary op dependent on function |
| `call` | `?` | Call function on top of stack |
| `def` |  | Assign value to variable symbol |
| `undef` |  | Unassign value from variable symbol, push value to stack |
| `for` |  | See below |
| `dip` | `_` | See below |
| `const` | `\|` | Specifies a symbol definition as a constant value |
`add`, `sub`, `mul`, `div`, `mod`, `eq`, `ne`, `gt`, `ge`, `lt`, `le`

## Parsing
```
# Some rules
:[a-zA-Z][_0-9a-zA-Z]* ⇒ Symbol (literal one-word string)
\$[0-9a-zA-Z][_0-9a-zA-Z]* ⇒ Temp variable
[a-zA-Z][_0-9a-zA-Z]* ⇒ Variable name
'{anything} ⇒ Literal character

# Some builtins more preprocessor-directives
STACK N fn loop ⇒ STACK fn? fn? ... fn? [N times]
-> x   y ⇒ {y} :x def
-> x | y ⇒   y :x def
-- ⇒ comment

# Other
\n ⇒ Literal newline is a token, other whitespace ignored/unimportant
-> x (:word :list) { y }-- def ⇒ Specify temp words to unassign after. Acts as guard.
```
