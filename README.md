# Night
**(WIP)** Left-to-right interpreted concatenative stack-based programming language.
Kinda just writing this for fun right now. Not planning on it being intended for golfing,
but considering I mostly wrote languages for this it may have some inspired elements.

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
		$for_f ?
		:I undefr $for_r
	} loop
	:for_r undef ;
}
```
## Some Definitions
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
## Impl Order
1. Basic expression parsing
2. Math and equality checking
3. Arrays
4. Variable assignment
5. Functions on stack
6. Loops
7. Figure it out from there

## Conversion rep
```rust
// 5 :x ! . * $x : - 5

[Number(5), Word("x"), Op(TmpAssign), Op(Dup), Op(Mul), Register("x"), Op(Swap), Op(Sub), Number(5)]

struct Value;
enum Instr {
	Push(Value), // push val
	PushFunc(Rc<BiFunction>), // push func
	PushSym(String, bool), // push symbol
	Op(Operator), // Run operator
	Call(BiFunction), // call func
	Guard(Vec<String>), // temp reg guard
	Drop(Vec<String>), // temp reg drop
	BeginArray, // beginning of array
	EndArray, // end of array
}
[Push(5), Push("x"), Op(TmpAssign), Op(Dup), Op(Mul), PushSym("x", true), Op(Swap), Op(Sub), Push(5)]

// Pre-exec step to expand out `LitFunct` and `Variable` maybe? 

// Like this
trait InlineFunction;
impl InlineFunction for Dup {
	fn call(&self, scope: Scope) -> Result<Scope, NightError> {
		let val = scope.pop_stack()?;
		scope.push_stack(val.clone());
		scope.push_stack(val);
		Ok(scope)
	}
}

// 1 2 3 {+} _
[Push(1), Push(2), Push(3), OpenCurly, Op(Add), CloseCurly, Op(Dip)]

// Should have fns like as_prim or smth similar to help
struct BiFunction {
	contents: Vec<Instr>,
	guard: Vec<String>,
}
// Note: This definition should have issues since the contents are a list of unevaluated actions. Scope should have defs for variables, so either I do something like 
// ```
// let scope = Night::from_scope(scope).exec_instr(each self.contents);
// scope
// ```
// or somehow have the inner contents of a BiFunction split out into the main action list, and then run to avoid nesting? Unsure. If the latter, BiFunction shouldn't implement InlineFunction (see above)
impl<'a> InlineFunction for BiFunction<'a>;

// Temp register guarding example
// dip(:_dip_inter) <- : :_dip_inter ! ; ? $_dip_inter
[Value(1), Value(2), Value(3), LitFunct(BiFunction), Guard(["_dip_inter"]), Op(Dip), Drop(["_dip_inter"])]
```
