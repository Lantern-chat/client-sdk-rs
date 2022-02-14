Lantern Rust SDK
================

This crate provides all the structures required to interact with Lantern homeservers.

## Models

These are the raw struct definitions for objects.

## API

Contains structures and defitions for the REST API, abstracting into "Commands"

Commands are used within the `Driver` layer to perform actions.

## Driver

A low-level REST interface using Commands to accomplish tasks.

## Client

A thread-safe mid-level abstraction around the API.

## Gateway

WebSocket Gateway object

## Framework

Ready-made bot framework