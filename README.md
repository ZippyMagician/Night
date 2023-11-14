# Night
Left-to-right stack-based programming language

## Basic Syntax
```ruby
x <- 4 7 +
x x * print

mults  <- :a ! ; 11 1 range { $a I + } @
mults2 <- :a ! 9 { . $a + } loop
7 mults # [7 14 21 28 35 42 49 56 63 70]

dip <- :cache | : << ? >>
for <- "$for_r", "$for_fn" | {
	:for_fn ! ; . len : :for_r ! ;
	{
		$for_r . head : 1 drop :for_r ! ; :I := ;
		$for_fn ?
		:I :!= ;
	} loop
}
```
## Some Definitions
| Function | Symbol | Def |
| -- | -- | -- |
| `pop` | `;` | Pop top value from stack |
| `dup` | `.` | Duplicate top value of stack |
| `tmp` | `!` | Assign value to temp variable |
| `join` | `,` | Joins top two values from stack into array |
| `I` | | Intermediary op dependent on function |
| `call` | `?` | Call function on top of stack |
| `assign` | `:=` | Assign value to variable symbol |
| `unassign` | `:!=` | Unassign value from variable symbol, push value to stack |
| `cpush` | `<<` | Pop top value from stack and push it to the cache |
| `cpop` | `>>` | Pop top value from the cache and push it to the stack |
| `fork` | | `a b fork` equiv to `. a : b` |
| `for` | `@` | See below |
| `dip` | `_` | See below |
| `filter` | `$` | Self explanatory |
`add`, `sub`, `mul`, `div`, `mod`, `eq`, `ne`, `gt`, `ge`, `lt`, `le`
## Parsing
```
# Some rules
:[a-zA-Z][_0-9a-zA-Z]* ⇒ Symbol (literal one-word string)
\$[0-9a-zA-Z][_0-9a-zA-Z]* ⇒ Temp variable
[a-zA-Z][_0-9a-zA-Z]* ⇒ Variable name

# Some builtins more preprocessor-directives
STACK N fn loop ⇒ STACK fn fn ... fn [N times]
x <- y ⇒ y :x assign
-- ⇒ comment

# Other
| ⇒ Place guard on function for certain features/variables to prevent issues
\n ⇒ Literal newline is a token
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

[Number(5), Word("x"), Operator(TmpAssign), Operator(Dup), Operator(Mul), Register("x"), Operator(Swap), Operator(Sub), Number(5)]

struct Action<'a> {
	span: Span<'a>,
	rep: Node,
}
enum Node {
	Literal(Value),
	Variable(String, bool),
	Function(Rc<impl InlineFunction>),
	PopAction,
}
[Literal(5), Literal("x"), Function(TmpAssign), Function(Dup), Function(Mul), Variable("x", true), Function(Swap), Function(Sub), Literal(5)]

// Like this
impl InlineFunction for Dup {
	fn call(&self, scope: Scope) -> Result<Scope, Box<dyn ToString>> {
		let val = scope.pop_stack()?;
		scope.push_stack(val.clone());
		scope.push_stack(val);
		Ok(scope)
	}
}
```

