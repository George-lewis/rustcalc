# RustCalc

A CLI calculator written in rust

## Screenshot

![](screenshots/screenshot.png)

# Features

 - *Advanced Editing*: Rustcalc allows you to edit statements in-line and keeps a navigatable history of your input

## Bedmas

Rustcalc obeys the standard order of operations:

![](screenshots/bedmas.png)

## Expressivity

Rustcalc supports multiple representations of standard operations, including plain English!

> A full list of operators is available in the [Reference](tbd)

![](screenshots/expressivity.png)

## Constants

Rustcalc comes with some useful predefined constants:

![](screenshots/expressivity.png)

## Variables

You can define your own variables and use them in computation.

Variables are indicated by a `$` prefix, and can have any name.

![](screenshots/variables-1.png)

You can list all defined variables with `$`!

![](screenshots/variables-2.png)

`$ans` is defined for you, and takes on the value of the last statement!

![](screenshots/variables-3.png)

## RCFile

Rustcalc supports running a script at runtime. On first run, Rustcalc will generate a default RCFile.

On startup, Rustcalc will load the RCFile and run each line as if it had been input manually.

This allows you to define variables that will be available immediately.

On first startup:

```
RCFile doesn't exist. Creating default at [C:\Users\George\AppData\Roaming\rustcalc.rc]
```

Example RCFile:

```
// I use this a lot
$golden_ratio = 1.618033
```

`$golden_ratio` will then be created at startup and available for use.

# Reference