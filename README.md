# Crafting Interpreters in Rust

## Overview

This repository contains the source code for the [Crafting Interpreters](https://craftinginterpreters.com/) book, implemented in Rust. The book covers the design and implementation of a simple programming language, including lexical analysis, parsing, and evaluation.

The project implements most of challenges in the book:
- [ ] `/*...*/` block comments
- [x] Ternary operator `conditional ? expression : expression`
- [x] Support `+` operator for different object types such as `"scone" + 4`
- [x] Devide by zero error handling
- [x] Uninitialized variable error handling
- [x] `break`, `continue`, statements
- [x] Lambda/anonymous function
- [x] Bind variable accesses in local scope to outer scopes
- [x] Static, getter methods in a class
- [ ] `inner` method

The `tests/` folder contains unit tests for the implementation.

## Future development
The book covers 2 parts:
- Part 1: Interpreter
- Part 2: Compiler for a Virtual Machine

The project only finishes part 1. I'll complete part 2 later when I feel it's useful.

The project will use [Cargo Workspaces](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html) if I implement part 2.
