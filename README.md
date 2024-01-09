# Night
Stack-based concatenative language.

## About
**TODO:** Proper about section. Currently the README is mainly used for information storage for my future usage.

Note for future: Currently there is no way for builtins and operators to efficiently call block arguments (`call`, `loop`, `if`). The current solution is to make them exceptions called directly from `Night`, instead of defining them in `builtin/defs.rs` like the other operators and builtin symbols. It's not something vitally important to change, just something that is slightly clunky in my opinion. This most likely won't be changed however, as it would require a major overhaul to some parts of the code.

- [x] Lexer
- [x] Basic interpreter
- [x] Operator structure
- [x] Builtin structure
- [x] Const symbol definitions
- [x] Blocks
- [x] Symbol definitions
- [x] Register definitions
- [ ] Mutable scoped registers
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
> would function as an alternate definition syntax. In this version, `def` and `defr` would no longer exist. 
> 
> The drawbacks include the fact that you can no longer dynamically define symbols from strings (which was possible via `def` and `defr`), and that typos will be much harder to notice when it comes to symbols. However, this would definitely clean up some definitions in the code. 
> 
> For instance, the `for` defintion, as can be seen below, would look much cleaner. Additionally, the definition of registers will be much more apparent at first glance. However, since the pattern `:reg ! ;` has become `$reg`, `:reg !` must become `$reg $reg`, which looks clunky and is not ideal.
> 
> Below is a code snippet where `defr`/`!` has been ommitted. `:reg ! ;` has become `$reg ;`.
> ```ruby
> -> mults  (:a) $a ; 11 1 range { $a $I + } for
> -> mults2 (:a) $a 9 { . $a + } loop
> 
> -> dip (:top) : $top ; :top | ? $top
> -> for (:for_f) {
>     $for_f ; . len (:for_r :I) { . first : 1 drop $for_r ; $I ; $for_f [:for_f :for_r] | ? $for_r } loop
> }
> ```
> You can compare the definition of `dip` with the prior example in this feature idea, where register definition implicitly pops for comparison. If this feature is implemented, it remains to be seen which variant will be the standard. Most likely, despite the slight unintuitiveness in my opinion, it remains superior to the alternative `$a $a` pattern you would need for the `mults2` definition, where register definition does not preserve on the stack.
> 
> There also exists a possibility this change will exist solely for register definitions, and there be no alternative symbol definition besides `->` syntactically on implementation of this idea, which may help dodge the issue mentioned above wherein typos can be very difficult to track down.
> 
> Finally, this change would warrant a look at `undef` (and also possibly `undefr`) to see how that will be handled.

Implemented as follows:

```ruby
-> dip (:top) : :top ! ; :top | ? $top
```
is now
```ruby
-> dip (top) : $top! :top | ? $top
```
This makes it apparent when a register is being defined vs. pushed while also cleaning up messiness. As of now, `def` is staying, as is `undefr` and `undef`, all of which take string arguments. At some point `def` will probably be changed, although there isn't really a need for inline variable definition as registers exist.

However, allowing for mutable scoped registers at some point will most likely be important.

#### Other
- Complex number support
- More math builtins
- Imports
- Lower level operations

## Basic Syntax
```ruby
-> x| 4 7 +
x x * print

-> mults  (a) $a! 1 11 range { $a $I + } for
-> mults2 (a) . $a! 9 { . $a + } loop
7 mults

-> dip (top) : $top! :top | ? $top
-> for (for_f) {
	$for_f! . len
	(for_r I) {
		. first : 1 drop $for_r! $I!
		$for_f [:for_f :for_r] | ? $for_r
	} loop
}
```

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
-> x (word list) { y } ⇒ Specify temp words to unassign after. Acts as guard on registers.
-- ⇒ comment
```
