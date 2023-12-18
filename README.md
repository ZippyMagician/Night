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
- [x] Fully decide how arrays will work
- [ ] Implement basic array support
- [ ] Implement array support builtins + ops
- [x] Implement operators
- [x] Implement basic builtins (poc)
- [ ] Choose + Implement more useful builtins
- [x] Basic function composition boilerplate
- [ ] Implement basic function composition builtins + ops

### Considered features
#### _(Implemented)_ Private/Public Guard Distinctions
> For guards, maybe add a way to stop arbitrary blocks executed with the `call` op from using certain guarded values?
> Something like:
> ```ruby
> (:private1 :private2 | :public) { ... }
> ```
> This also gives more syntactic use for `|`, which currently is only used for const defs.
> If I did do this, I'd have to slightly redesign how guarded registers are stored. Currently they it's just a `HashSet<String>`, which wouldn't properly distinguish between the two types (and also the level they exist in). Maybe add a new instr marker placed after `call` ops pre-execution as a marker? Then have a separate `HashSet` for inaccessible registers. Probably the simplest solution.

Implemented in a slightly different manner that allows for more choices.
```hs
(:private1 :public) {
	-- ... do things
	:private1 | ?
	-- ... do more things
}
```
This will "block" the register *$private1* from being accessed in whatever is called by the `call` op. Once arrays are implemented, something like
```hs
[:private1 :private2] | ?
```
will work too.
#### Drop `def` and `defr`/`!`
> Definition of symbols and registers could be based on if it is undefined, i.e.,
> ```ruby
> -> dip (:top) : $top :top | ? $top
> ```
> will work, as `$top` starts undefined, meaning night will automatically assign the value to the register instead. In this version, `pop` is unecessary after the definition. Similarly,
> ```ruby
> (:top) { : $top :top | ? $top } dip
> ```
> would function as an alternate definition syntax. In this version, `def` and `defr` would no longer exist. The drawbacks include the fact that you can no longer dynamically define symbols from strings (which was possible via `def` and `defr`), and that typos will be much harder to notice when it comes to symbols. However, this would definitely clean up some definitions in the code. For instance, the `for` defintion found below would look much cleaner.

### Possible future ideas
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

-> dip (:top) : :top ! ; :top | ? $top
-> for (:for_f) {
	:for_f ! ; . len
	(:for_r :I) {
		. first : 1 drop :for_r ! ; :I ! ;
		$for_f [:for_f :for_r] | ? $for_r
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
\n ⇒ Literal newline is a token, other whitespace ignored/unimportant

# Some builtins more preprocessor-directives
-> x   y ⇒ {y} :x def
-> x | y ⇒   y :x def
:x | <instr>       ⇒ Block register x from being access for the duration of the next instr
[:x :y] | <instr>  ⇒ Block register[s] x & y from being accessed for the duration of the next instr
-> x (:word :list) { y } ⇒ Specify temp words to unassign after. Acts as guard on registers.
-- ⇒ comment
```
