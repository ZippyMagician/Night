# Night
**(WIP)** Interpreted concatenative stack-based language.

## About
TODO. Currently working on getting the major stuff implemented before I rewrite the readme and work on documentation.

Note for future: Currently there is no way for builtins and operators to efficiently call block arguments. The current solution is to offload their definitions to the actual `Night` struct and leave the definitions in `defs.rs` as internal errors (since they should never be called). It's not like it's any slower to do it this way, I just don't really like it right now, as it seems clunky in some situations. Might be a better way.

- [x] Lexer
- [x] Basic interpreter
- [x] Operator structure
- [x] Builtin structure
- [x] Const symbol definitions
- [x] Blocks
- [x] Symbol definitions
- [x] Register definitions
- [x] Guard statements
- [ ] Fully decide how arrays will work
- [ ] Implement basic arrays
- [ ] Implement array support ops
- [x] Implement operators
- [x] Implement basic builtins (poc)
- [ ] Choose + Implement more useful builtins

For guards, maybe add a way to stop arbitrary blocks executed with the `call` op from using certain guarded values?
Something like:
```ruby
(:private1 :private2 | :public) { ... }
```
This also gives more syntactic use for `|`, which currently is only used for const defs.
If I did do this, I'd have to slightly redesign how guarded registers are stored. Currently they it's just a `HashSet<String>`, which wouldn't properly distinguish between the two types (and also the level they exist in). Maybe add a new instr marker placed after `call` ops pre-execution as a marker? Then have a separate `HashSet` for inaccessible registers. Probably the simplest solution.

Possible ideas
- Complex number support
- More math builtins
- Imports
- Lower level operations

## Basic Syntax
```ruby
-> x| 4 7 +
x x * print

-> mults  (:a) :a ! ; 11 1 range { $a $I + } for
-> mults2 (:a) :a ! 9 { . $a + } loop
7 mults

-> dip (:top) : :top ! ; ? $top
-> for (:for_f) {
	:for_f ! ; . len
	(:for_r :I) {
		. first : 1 drop :for_r ! ; :I ! ;
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
