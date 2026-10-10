# Grammar

## Variable declaration

`let` and `const` will both be used for variable declaration

### Binding
`bind` will be `name[: ty]`
### Mutable variable
`let bind [= expr][;|EOL]`

### Constants
Is the same as `let` but with constant and only can be evaluated when the variables values are set not ambiguous
and only with value
`const bind = expr[;|EOL]`

## Function declaration
`fn name[:ty]\[bind[, bind]...\](bind[, bind]...) {...}`

### Return statement
`return expr;`
`expr` is required
So return won't accept empty `expr`
if you want to indicate a `void`
or no return type just end last statement with semicolon

## Control Flow

### If condition

`if stmt {..}`
### Else
`if stmt {..} else {...}`

### Else-if
`if stmt {..} else if stmt2 {...}`

### Pattern aka match
Some and None
```
pat expr {
  Some(d) -> {}
  None -> {}
}
```

Patterns

```
pat expr {
  ident if condition -> {
    ...
  }
  pattern1 | (pattern2 & pattern3) -> {}
  _ -> {}
}
```



# Construction

Units: are a small functions that just do a mini-task that will be the core of reusable functions
## Units

Line-Terminator unit:
Will search for line terminator `[;]\n`

Argument-Terminator unit:
Will search for argument/parameter terminators `[,])`

Parameter-Terminator unit:
Will search for argument/parameter terminators `[,]\]`

Declaration unit:
Will search for this pattern `name[: type]`

Value-eval unit:
Will search for `= value[;]`

Function-eval unit:
Will search for `\[[arg1, arg2, ...]\]([arg1, arg2, ...]) {...}`
