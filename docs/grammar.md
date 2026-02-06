# G-Basic EBNF Grammar

This document provides the complete Extended Backus-Naur Form (EBNF) grammar specification for the G-Basic programming language.

## Notation Conventions

- `::=` defines a production rule
- `|` separates alternatives
- `()` groups elements
- `[]` denotes optional elements (0 or 1)
- `{}` denotes repetition (0 or more)
- `"..."` denotes terminal symbols (literal text)
- All keywords and identifiers are case-insensitive
- Whitespace and comments are implicitly allowed between tokens

## Lexical Elements

### Comments

```ebnf
comment ::= line_comment | block_comment
line_comment ::= "//" { any_char_except_newline } newline
block_comment ::= "/*" { any_char } "*/"
```

### Identifiers

```ebnf
identifier ::= letter { letter | digit | "_" }
letter ::= "a".."z" | "A".."Z" | "_"
digit ::= "0".."9"
```

Note: Identifiers and keywords are normalized to lowercase during lexical analysis.

### Keywords

```ebnf
keyword ::= "let" | "fn" | "if" | "else" | "for" | "in" | "while"
          | "match" | "return" | "break" | "continue"
          | "true" | "false"
          | "int" | "float" | "string" | "bool" | "void"
```

### Namespace Keywords

```ebnf
namespace ::= "screen" | "sound" | "input" | "math"
            | "system" | "memory" | "io"
```

### Literals

```ebnf
literal ::= integer | float | string | boolean

integer ::= digit { digit }

float ::= digit { digit } "." digit { digit } [ exponent ]
exponent ::= ( "e" | "E" ) [ "+" | "-" ] digit { digit }

string ::= '"' { string_char | escape_sequence } '"'
string_char ::= any_char_except_quote_or_backslash
escape_sequence ::= "\\" ( "n" | "t" | "r" | "\\" | '"' | "0" )

boolean ::= "true" | "false"
```

### Operators

```ebnf
binary_op ::= "+" | "-" | "*" | "/" | "%"
            | "==" | "!=" | "<" | ">" | "<=" | ">="
            | "&&" | "||"

unary_op ::= "!" | "-"

assignment_op ::= "="
```

### Delimiters

```ebnf
delimiter ::= "(" | ")" | "{" | "}" | "[" | "]"
            | "," | "." | ":" | ";" | "->"
```

## Program Structure

```ebnf
program ::= { statement }
```

## Statements

```ebnf
statement ::= let_statement
            | function_declaration
            | if_statement
            | for_statement
            | while_statement
            | match_statement
            | return_statement
            | break_statement
            | continue_statement
            | block
            | expression_statement

statement_terminator ::= newline | ";" | EOF
```

### Let Statement

```ebnf
let_statement ::= "let" identifier [ ":" type ] "=" expression statement_terminator
```

### Function Declaration

```ebnf
function_declaration ::= "fn" identifier "(" parameter_list ")" [ "->" type ] block

parameter_list ::= [ parameter { "," parameter } ]
parameter ::= identifier ":" type
```

### If Statement

```ebnf
if_statement ::= "if" expression block [ "else" block ]
```

### For Statement

```ebnf
for_statement ::= "for" identifier "in" expression block
```

### While Statement

```ebnf
while_statement ::= "while" expression block
```

### Match Statement

```ebnf
match_statement ::= "match" expression "{" { match_arm } "}"
match_arm ::= pattern "->" block [ "," ]
```

### Return Statement

```ebnf
return_statement ::= "return" [ expression ] statement_terminator
```

### Break Statement

```ebnf
break_statement ::= "break" statement_terminator
```

### Continue Statement

```ebnf
continue_statement ::= "continue" statement_terminator
```

### Block

```ebnf
block ::= "{" { statement } "}"
```

### Expression Statement

```ebnf
expression_statement ::= expression statement_terminator
```

## Expressions

Expressions use precedence climbing for binary operators. Listed from lowest to highest precedence:

```ebnf
expression ::= assignment_expression

assignment_expression ::= logical_or_expression [ "=" assignment_expression ]

logical_or_expression ::= logical_and_expression { "||" logical_and_expression }

logical_and_expression ::= equality_expression { "&&" equality_expression }

equality_expression ::= comparison_expression { ( "==" | "!=" ) comparison_expression }

comparison_expression ::= additive_expression { ( "<" | ">" | "<=" | ">=" ) additive_expression }

additive_expression ::= multiplicative_expression { ( "+" | "-" ) multiplicative_expression }

multiplicative_expression ::= unary_expression { ( "*" | "/" | "%" ) unary_expression }

unary_expression ::= ( "!" | "-" ) unary_expression
                   | postfix_expression

postfix_expression ::= primary_expression { postfix_operator }

postfix_operator ::= function_call
                   | index_access
                   | field_access

function_call ::= "(" argument_list ")"

index_access ::= "[" expression "]"

field_access ::= "." identifier

primary_expression ::= literal
                     | identifier
                     | method_chain
                     | array_literal
                     | parenthesized_expression

parenthesized_expression ::= "(" expression ")"

array_literal ::= "[" argument_list "]"

argument_list ::= [ expression { "," expression } ]
```

### Method Chains

Method chains are special expressions that start with a namespace and chain method calls:

```ebnf
method_chain ::= namespace { method_call }

method_call ::= "." identifier "(" argument_list ")"
```

Note: A method chain must have at least one method call (e.g., `Screen.Layer(0)`).

## Patterns

Patterns are used in match statements:

```ebnf
pattern ::= literal
          | identifier
          | wildcard

wildcard ::= "_"
```

## Types

```ebnf
type ::= primitive_type
       | array_type

primitive_type ::= "int" | "float" | "string" | "bool" | "void"

array_type ::= "[" type "]"
```

### Opaque Handle Types

The type system also includes opaque handle types for runtime resources. These are not directly constructible in source code but are returned by namespace methods:

- `Sprite` - Graphics sprite handle
- `Layer` - Graphics layer handle
- `Sound` - Sound effect handle
- `Instrument` - Musical instrument handle
- `Timer` - Timer handle

## Operator Precedence

From lowest to highest precedence:

1. Assignment: `=` (right-associative)
2. Logical OR: `||` (left-associative)
3. Logical AND: `&&` (left-associative)
4. Equality: `==`, `!=` (left-associative)
5. Comparison: `<`, `>`, `<=`, `>=` (left-associative)
6. Additive: `+`, `-` (left-associative)
7. Multiplicative: `*`, `/`, `%` (left-associative)
8. Unary: `!`, `-` (prefix)
9. Postfix: `.`, `[]`, `()` (left-associative)

## Examples

### Variable Declaration

```gbasic
let x = 42
let y: Float = 3.14
let name: String = "Alice"
```

### Function Declaration

```gbasic
fn add(a: Int, b: Int) -> Int {
    return a + b
}

fn greet(name: String) {
    IO.Print("Hello, " + name)
}
```

### Control Flow

```gbasic
if x > 10 {
    IO.Print("Large")
} else {
    IO.Print("Small")
}

for i in [1, 2, 3, 4, 5] {
    IO.Print(i)
}

while x < 100 {
    x = x * 2
}

match x {
    0 -> { IO.Print("Zero") }
    1 -> { IO.Print("One") }
    _ -> { IO.Print("Other") }
}
```

### Method Chains

```gbasic
Screen.Layer(0).Sprite("hero").Draw()
Screen.Layer(1).Rect(10, 20, 100, 50).Fill(255, 0, 0)
Sound.PlayEffect("explosion").Volume(0.8)
```

### Arrays and Indexing

```gbasic
let numbers = [1, 2, 3, 4, 5]
let first = numbers[0]
numbers[2] = 99
```

### Expressions

```gbasic
let result = (x + y) * z
let is_valid = x > 0 && y < 100
let color = Math.Clamp(value, 0, 255)
```

## Whitespace and Newlines

- Whitespace (spaces, tabs) is generally insignificant and used only for token separation
- Newlines can act as statement terminators (alternative to semicolons)
- Multiple consecutive newlines are treated as a single terminator
- Newlines inside parentheses, brackets, or braces are ignored
- Blank lines and indentation have no syntactic meaning

## Case Insensitivity

G-Basic is case-insensitive for keywords and identifiers:

```gbasic
LET X = 42      // equivalent to: let x = 42
IF x > 0 {      // equivalent to: if x > 0 {
    RETURN X    // equivalent to: return x
}
```

String literals preserve case:

```gbasic
let name = "Alice"  // "Alice" != "ALICE"
```
