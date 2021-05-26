<div align="center">
  <h1>Go Away</h1>

  <p>
    <strong>Generate Go Types from Rust Types</strong>
  </p>

  <p>
    <a href="https://crates.io/crates/go-away"><img alt="Crate Info" src="https://img.shields.io/crates/v/go-away.svg"/></a>
    <a href="https://docs.rs/go-away/"><img alt="API Docs" src="https://img.shields.io/badge/docs.rs-go-away-green"/></a>
  </p>

  <h4>
    <a href="https://github.com/obmarg/go-away/blob/master/CHANGELOG.md">Changelog</a>
  </h4>
</div>

# Overview

Go Away is a small library for generating go types & marshalling code from Rust
type definitions.  It's intended for use when you have existing rust code that
is using serde for JSON serialization and you want to allow go services or
clients to interact with that code.

It may be expanded to other languages at some point but it's mostly been built
to service a very specific need and might never evolve past that.
