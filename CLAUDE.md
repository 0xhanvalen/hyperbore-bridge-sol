# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build/Test/Lint Commands

- **Run all tests**: `yarn test` or `anchor test`
- **Run single test**: `yarn run ts-mocha -p ./tsconfig.json -t 1000000 tests/specific-test.ts`
- **Lint code**: `yarn lint`
- **Fix linting issues**: `yarn lint:fix`
- **Build**: `anchor build`
- **Deploy**: `ts-node migrations/deploy.ts`

## Locations

To save on typing time, we'll define 3 file locations and short-names for the files:
smart-contract: programs/bridge-sol/src/lib.rs
tests: tests/bridge-sol.ts
audit: AUDIT.md

## Code Style Guidelines

- **Rust**: Follow standard Rust formatting with descriptive function/variable names and error handling via Result<>
- **TypeScript**: Use explicit types for all parameters and return values
- **Imports**: Group imports by source (external crates first, then internal modules)
- **Error Handling**: Use the `error!()` macro with appropriate ErrorCode enums
- **Naming Conventions**:
  - Rust: snake_case for functions/variables, CamelCase for types/structs
  - TypeScript: camelCase for variables/functions, PascalCase for classes/types
- **Comments**: Document complex logic and cryptographic operations
- **Testing**: Each instruction should have corresponding tests

## Security Notes

- Always validate all inputs thoroughly
- Protect against arithmetic overflow using checked operations
- Properly verify cryptographic signatures
